#![deny(clippy::all)]

use anyhow::Result;
use darkredis::ConnectionPool as RedisPool;
use postgres::{Client as PgClient, NoTls};
use sqlx::postgres::PgPool;
use tokio::{runtime::Runtime, stream::StreamExt};
use twilight_cache_inmemory::InMemoryCache;
use twilight_gateway::Cluster;
use twilight_http::Client as HttpClient;
use twilight_model::gateway::Intents;
use twilight_standby::Standby;

use crate::model::Context;

mod checks;
mod commands;
mod handler;
mod jobs;
mod logger;
mod migrations;
mod model;
mod reactions;
mod table;

async fn run_bot() -> Result<()> {
    // configure shard cluster
    let cluster = Cluster::builder(
        &dotenv::var("TOKEN")?,
        Intents::GUILD_MESSAGES | Intents::GUILDS | Intents::GUILD_MESSAGE_REACTIONS,
    )
    .build()
    .await?;

    // create the primary parental context, with new instances of all members
    let pool = PgPool::builder().max_size(8).build(&dotenv::var("DATABASE_URL")?).await?;
    let redis = RedisPool::create((&dotenv::var("REDIS")?).into(), None, 4).await?;

    let context = Context {
        cache: InMemoryCache::new(),
        http: HttpClient::new(&dotenv::var("TOKEN")?),
        pool,
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
        let mut sync_client = PgClient::configure()
            .user(&dotenv::var("POSTGRES_USER")?)
            .dbname(&dotenv::var("POSTGRES_DBNAME")?)
            .host(&dotenv::var("POSTGRES_HOST")?)
            .connect(NoTls)?;

        migrations::migrations::runner().run(&mut sync_client)?;
    }

    let mut rt = Runtime::new()?;
    rt.block_on(run_bot())?;

    Ok(())
}
