use darkredis::ConnectionPool as RedisPool;
use sqlx::postgres::PgPool;
use twilight::{
    cache::InMemoryCache,
    http::{error::Result as HttpResult, Client as HttpClient},
    model::{
        channel::{Message, ReactionType},
        gateway::payload::{MessageCreate, ReactionAdd},
    },
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
}

#[derive(Debug)]
pub struct MessageContext {
    pub cache: InMemoryCache,
    pub http: HttpClient,
    pub pool: PgPool,
    pub redis: RedisPool,
    pub standby: Standby,
    pub message: Box<MessageCreate>,
    pub content: String,
}

impl MessageContext {
    pub fn new(context: EventContext, message: Box<MessageCreate>, content: String) -> Self {
        Self {
            cache: context.cache,
            http: context.http,
            pool: context.pool,
            redis: context.redis,
            standby: context.standby,
            message,
            content,
        }
    }

    pub fn tokenized(self: &Self) -> Vec<String> {
        let mut tokens = Vec::new();

        let mut inside = false;
        let mut token = String::new();
        for c in self.content.chars() {
            if c == ' ' && !inside {
                tokens.push(token.to_owned());
                token = String::new();
                continue;
            }

            if c == '"' {
                inside = !inside;
            } else {
                token.push(c);
            }
        }
        tokens.push(token.to_owned());

        tokens
    }

    pub async fn reply(self: &Self, content: impl Into<String>) -> HttpResult<Message> {
        self.http.create_message(self.message.channel_id).content(content).unwrap().await
    }

    pub async fn react(self: &Self, emoji: impl Into<String>) -> HttpResult<()> {
        let emoji = ReactionType::Unicode { name: emoji.into() };
        self.http.create_reaction(self.message.channel_id, self.message.id, emoji).await
    }
}

#[derive(Debug)]
pub struct ReactionContext {
    pub cache: InMemoryCache,
    pub http: HttpClient,
    pub pool: PgPool,
    pub redis: RedisPool,
    pub reaction: Box<ReactionAdd>,
}

impl ReactionContext {
    pub fn new(context: EventContext, reaction: Box<ReactionAdd>) -> Self {
        Self {
            cache: context.cache,
            http: context.http,
            pool: context.pool,
            redis: context.redis,
            reaction,
        }
    }
}
