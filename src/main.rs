mod checks;
mod commands;
mod handler;
mod jobs;
mod logger;
mod model;
mod table;

use crate::model::BaseContext;
use darkredis::ConnectionPool as RedisPool;
use futures_util::stream::StreamExt;
use hyper::Client as HyperClient;
use hyper_rustls::HttpsConnector;
use sqlx::PgPool;
use std::sync::Arc;
use twilight_cache_inmemory::InMemoryCache;
use twilight_gateway::Cluster;
use twilight_http::Client as HttpClient;
use twilight_model::gateway::Intents;
use twilight_standby::Standby;

#[deny(clippy::all)]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // load dotenv and logger
    dotenv::dotenv()?;
    pretty_env_logger::init();

    // configure sqlx
    let postgres = PgPool::connect(&dotenv::var("DATABASE_URL")?).await?;

    // run migrations
    sqlx::migrate!("./migrations/").run(&postgres).await?;

    // configure shard cluster
    let (cluster, mut events) = Cluster::builder(
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

    // connect to redis
    let redis = RedisPool::create((&dotenv::var("REDIS")?).into(), None, 4).await?;

    // build the hyper client
    let https = HttpsConnector::with_native_roots();
    let hyper = HyperClient::builder().build(https);

    // create the primary parental context, with new instances of all members
    let context = BaseContext::new(
        InMemoryCache::new(),
        HttpClient::new(dotenv::var("TOKEN")?),
        hyper,
        postgres,
        redis,
        Standby::new(),
    );

    // start the cluster in the background
    let cluster_spawn = Arc::new(cluster);
    tokio::spawn(async move {
        cluster_spawn.up().await;
    });

    // start jobs
    tokio::spawn(jobs::start(context.clone()));

    // listen for events
    while let Some((_, event)) = events.next().await {
        context.cache().update(&event);
        context.standby().process(&event);

        tokio::spawn(handler::event(event, context.clone()));
    }

    Ok(())
}
