use anyhow::Result;
use dotenv;
use postgres::{Client, NoTls};
use pretty_env_logger;
use sqlx::postgres::PgPool;
use tokio::{runtime::Runtime, stream::StreamExt};
use twilight::{
    cache::InMemoryCache,
    gateway::{Cluster, ClusterConfig},
    http::Client as HttpClient,
    model::gateway::GatewayIntents,
};

use crate::model::EventContext;

mod commands;
mod handler;
mod migrations;
mod model;
mod table;
mod util;

async fn run_bot() -> Result<()> {
    // connect to a postgres pool
    let pool = PgPool::builder()
        .max_size(8)
        .build(&dotenv::var("DATABASE_URL")?)
        .await?;

    // create and start bot
    let cluster_config = ClusterConfig::builder(&dotenv::var("TOKEN")?)
        .intents(Some(
            GatewayIntents::GUILD_MESSAGES
                | GatewayIntents::GUILDS
                | GatewayIntents::GUILD_MESSAGE_REACTIONS,
        ))
        .build();

    let cluster = Cluster::new(cluster_config);
    let cache = InMemoryCache::new();
    let http = HttpClient::new(&dotenv::var("TOKEN")?);

    cluster.up().await?;

    let mut events = cluster.events().await;
    while let Some(event) = events.next().await {
        cache.update(&event.1).await?;

        tokio::spawn(handler::handle_event(EventContext {
            cache: cache.clone(),
            http: http.clone(),
            pool: pool.clone(),
            event: event.1,
            id: event.0,
        }));
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    // load dotenv and logger
    dotenv::dotenv()?;
    pretty_env_logger::init();

    // run migrations
    let mut client = Client::configure()
        .user(&dotenv::var("POSTGRES_USER")?)
        .dbname(&dotenv::var("POSTGRES_DBNAME")?)
        .host(&dotenv::var("POSTGRES_HOST")?)
        .connect(NoTls)?;

    migrations::migrations::runner().run(&mut client)?;

    let mut rt = Runtime::new()?;
    rt.block_on(run_bot())?;

    Ok(())
}
