use dotenv;
use pretty_env_logger;
use tokio::stream::StreamExt;
use twilight::{
    cache::InMemoryCache,
    gateway::{Cluster, ClusterConfig},
    http::Client as HttpClient,
    model::gateway::GatewayIntents,
};

mod commands;
mod handler;
mod model;
mod util;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // load dotenv and logger
    dotenv::dotenv().ok();
    pretty_env_logger::init();

    // create and start bot
    let token = dotenv::var("TOKEN")?;
    let token = token.as_str();
    let http = HttpClient::new(token);
    let cache = InMemoryCache::new();

    let cluster_config = ClusterConfig::builder(token)
        .intents(Some(GatewayIntents::GUILD_MESSAGES))
        .build();
    let cluster = Cluster::new(cluster_config);
    cluster.up().await?;

    let mut events = cluster.events().await;
    while let Some(event) = events.next().await {
        cache.update(&event.1).await?;
        tokio::spawn(handler::handle_event(event, http.clone()));
    }

    Ok(())
}
