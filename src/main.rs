#![deny(clippy::all)]

use anyhow::Result;
use darkredis::ConnectionPool as RedisPool;
use postgres::{Client as PgClient, NoTls};
use sqlx::postgres::PgPool;
use tokio::{runtime::Runtime, stream::StreamExt};
use twilight::{
    cache::InMemoryCache,
    gateway::{Cluster, ClusterConfig},
    http::Client as HttpClient,
    model::gateway::GatewayIntents,
    standby::Standby,
};

use crate::model::Context;

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
    let cluster_config = ClusterConfig::builder(&dotenv::var("TOKEN")?)
        .intents(Some(
            GatewayIntents::GUILD_MESSAGES
                | GatewayIntents::GUILDS
                | GatewayIntents::GUILD_MESSAGE_REACTIONS,
        ))
        .build();

    let cluster = Cluster::new(cluster_config).await?;

    // create the primary parental context, with new instances of all members
    let context = Context {
        cache: InMemoryCache::new(),
        http: HttpClient::new(&dotenv::var("TOKEN")?),
        pool: PgPool::builder().max_size(8).build(&dotenv::var("DATABASE_URL")?).await?,
        redis: RedisPool::create((&dotenv::var("REDIS")?).into(), None, 4).await?,
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
    let mut events = cluster.events().await;
    while let Some((_, event)) = events.next().await {
        context.cache.update(&event).await?;
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
