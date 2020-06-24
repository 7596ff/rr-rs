use darkredis::ConnectionPool as RedisPool;
use sqlx::postgres::PgPool;
use twilight::{
    cache::InMemoryCache,
    gateway::Event,
    http::{error::Result as HttpResult, Client as HttpClient},
    model::channel::{Message, Reaction, ReactionType},
    standby::Standby,
};

#[derive(Debug)]
pub enum Response {
    Message(Message),
    Reaction,
    None,
}

#[derive(Debug)]
pub struct EventContext {
    pub cache: InMemoryCache,
    pub http: HttpClient,
    pub pool: PgPool,
    pub redis: RedisPool,
    pub standby: Standby,
    pub event: Event,
    pub id: u64,
}

#[derive(Debug)]
pub struct MessageContext {
    pub cache: InMemoryCache,
    pub http: HttpClient,
    pub pool: PgPool,
    pub redis: RedisPool,
    pub standby: Standby,
    pub content: String,
    pub message: Message,
}

impl MessageContext {
    pub async fn reply(self: &Self, content: impl Into<String>) -> HttpResult<Message> {
        self.http
            .create_message(self.message.channel_id)
            .content(content)
            .unwrap()
            .await
    }

    pub async fn react(self: &Self, emoji: impl Into<String>) -> HttpResult<()> {
        let emoji = ReactionType::Unicode { name: emoji.into() };
        self.http
            .create_reaction(self.message.channel_id, self.message.id, emoji)
            .await
    }
}

#[derive(Debug)]
pub struct ReactionContext {
    pub cache: InMemoryCache,
    pub http: HttpClient,
    pub pool: PgPool,
    pub redis: RedisPool,
    pub reaction: Reaction,
}
