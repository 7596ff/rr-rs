use darkredis::ConnectionPool as RedisPool;
use sqlx::postgres::PgPool;
use twilight::{
    cache::InMemoryCache,
    gateway::Event,
    http::{error::Error, Client as HttpClient},
    model::channel::{Message, Reaction, ReactionType},
};

#[derive(Debug)]
pub enum Response {
    Some(Message),
    Err(Error),
    Reaction(ReactionType),
    None,
}

#[derive(Debug)]
pub struct EventContext {
    pub cache: InMemoryCache,
    pub http: HttpClient,
    pub pool: PgPool,
    pub redis: RedisPool,
    pub event: Event,
    pub id: u64,
}

#[derive(Debug)]
pub struct MessageContext {
    pub cache: InMemoryCache,
    pub http: HttpClient,
    pub pool: PgPool,
    pub redis: RedisPool,
    pub content: String,
    pub message: Message,
}

#[derive(Debug)]
pub struct ReactionContext {
    pub cache: InMemoryCache,
    pub http: HttpClient,
    pub pool: PgPool,
    pub redis: RedisPool,
    pub reaction: Reaction,
}
