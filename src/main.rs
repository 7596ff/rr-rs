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

use crate::model::EventContext;

mod commands;
mod handler;
mod logger;
mod migrations;
mod model;
mod reactions;
mod table;

async fn run_bot() -> Result<()> {
    // connect to a postgres pool
    let pool = PgPool::builder().max_size(8).build(&dotenv::var("DATABASE_URL")?).await?;

    // connect to a redis pool
    let redis = RedisPool::create((&dotenv::var("REDIS")?).into(), None, 4).await?;

    // create and start bot
    let cluster_config = ClusterConfig::builder(&dotenv::var("TOKEN")?)
        .intents(Some(
            GatewayIntents::GUILD_MESSAGES
                | GatewayIntents::GUILDS
                | GatewayIntents::GUILD_MESSAGE_REACTIONS,
        ))
        .build();

    let cluster = Cluster::new(cluster_config).await?;
    let cache = InMemoryCache::new();
    let http = HttpClient::new(&dotenv::var("TOKEN")?);
    let standby = Standby::new();

    // start the cluster in the background
    let cluster_spawn = cluster.clone();
    tokio::spawn(async move {
        cluster_spawn.up().await;
    });

    // listen for events
    let mut events = cluster.events().await;
    while let Some((_, event)) = events.next().await {
        cache.update(&event).await?;
        standby.process(&event);

        tokio::spawn(handler::event(
            event,
            EventContext {
                cache: cache.clone(),
                http: http.clone(),
                pool: pool.clone(),
                redis: redis.clone(),
                standby: standby.clone(),
            },
        ));
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
