mod checks;
mod commands;
mod handler;
mod jobs;
mod logger;
mod migrations;
mod model;
mod reactions;
mod table;

use crate::model::BaseContext;
use anyhow::Result;
use darkredis::ConnectionPool as RedisPool;
use futures_util::stream::StreamExt;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio_postgres::{Config as PgConfig, NoTls};
use twilight_cache_inmemory::InMemoryCache;
use twilight_gateway::Cluster;
use twilight_http::Client as HttpClient;
use twilight_model::gateway::Intents;
use twilight_standby::Standby;

#[deny(clippy::all)]

async fn run_bot() -> Result<()> {
    // configure shard cluster
    let cluster = Cluster::builder(
        &dotenv::var("TOKEN")?,
        Intents::GUILDS
            | Intents::GUILD_MEMBERS
            | Intents::GUILD_EMOJIS
            | Intents::GUILD_INVITES
            | Intents::GUILD_MESSAGES
            | Intents::GUILD_MESSAGE_REACTIONS,
    )
    .build()
    .await?;

    // connect to postgres
    let (postgres, postgres_connection) = PgConfig::new()
        .user(&dotenv::var("POSTGRES_USER")?)
        .dbname(&dotenv::var("POSTGRES_DBNAME")?)
        .host(&dotenv::var("POSTGRES_HOST")?)
        .connect(NoTls)
        .await?;

    tokio::spawn(async move {
        if let Err(why) = postgres_connection.await {
            eprintln!("postgres connection error: {}", why);
        }
    });

    // connect to redis
    let redis = RedisPool::create((&dotenv::var("REDIS")?).into(), None, 4).await?;

    // create the primary parental context, with new instances of all members
    let context = BaseContext {
        cache: InMemoryCache::new(),
        http: HttpClient::new(&dotenv::var("TOKEN")?),
        postgres: Arc::new(postgres),
        redis,
        standby: Standby::new(),
    };

    // start the cluster in the background
    let cluster_spawn = cluster.clone();
    tokio::spawn(async move {
        cluster_spawn.up().await;
    });

    // start jobs
    tokio::spawn(jobs::start(context.clone()));

    // listen for events
    let mut events = cluster.events();
    while let Some((_, event)) = events.next().await {
        context.cache.update(&event);
        context.standby.process(&event);

        tokio::spawn(handler::event(event, context.clone()));
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    // load dotenv and logger
    dotenv::dotenv()?;
    pretty_env_logger::init();

    {
        // run migrations, and drop the client
        let mut sync_client = postgres::Client::configure()
            .user(&dotenv::var("POSTGRES_USER")?)
            .dbname(&dotenv::var("POSTGRES_DBNAME")?)
            .host(&dotenv::var("POSTGRES_HOST")?)
            .connect(postgres::NoTls)?;

        migrations::migrations::runner().run(&mut sync_client)?;
    }

    let rt = Runtime::new()?;
    rt.block_on(run_bot())?;

    Ok(())
}
