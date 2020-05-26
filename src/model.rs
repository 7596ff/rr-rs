use sqlx::postgres::PgPool;
use twilight::{
    cache::InMemoryCache,
    gateway::shard::Event,
    http::{error::Error, Client as HttpClient},
    model::channel::Message,
};

#[derive(Debug)]
pub enum Response {
    Some(Message),
    Err(Error),
    None,
}

#[derive(Debug)]
pub struct EventContext {
    pub cache: InMemoryCache,
    pub http: HttpClient,
    pub pool: PgPool,
    pub event: Event,
    pub id: u64,
}

#[derive(Debug)]
pub struct MessageContext {
    pub cache: InMemoryCache,
    pub http: HttpClient,
    pub pool: PgPool,
    pub content: String,
    pub message: Message,
}
