use anyhow::Result;
use darkredis::ConnectionPool as RedisPool;
use sqlx::postgres::PgPool;
use twilight::{
    cache::InMemoryCache,
    http::{error::Result as HttpResult, Client as HttpClient},
    model::{
        channel::{Message, ReactionType},
        gateway::payload::{MessageCreate, ReactionAdd},
        user::User,
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
    pub args: Vec<String>,
    pub content: String,
}

impl MessageContext {
    pub fn new(context: EventContext, message: Box<MessageCreate>) -> Result<Self> {
        let args = shellwords::split(&message.content)?;
        let content = args.clone().join(" ");

        Ok(Self {
            cache: context.cache,
            http: context.http,
            pool: context.pool,
            redis: context.redis,
            standby: context.standby,
            message,
            args,
            content,
        })
    }

    pub async fn reply(self: &Self, content: impl Into<String>) -> HttpResult<Message> {
        self.http.create_message(self.message.channel_id).content(content).unwrap().await
    }

    pub async fn react(self: &Self, emoji: impl Into<String>) -> HttpResult<()> {
        let emoji = ReactionType::Unicode { name: emoji.into() };
        self.http.create_reaction(self.message.channel_id, self.message.id, emoji).await
    }

    pub async fn find_member(self: &Self) -> Result<Option<User>> {
        if !self.message.mentions.is_empty() {
            let user = self.message.mentions.values().next().unwrap();
            return Ok(Some(user.to_owned()));
        }

        // TODO: wait for CachedGuild.members
        //
        // let guild_id = context.message.guild_id.ok_or(FindMemberError::NoGuild)?;
        // let guild = context.cache.guild(guild_id).await?;

        // let found = members
        //     .iter()
        //     .find(|&member| member.display_name().into_owned() == search_str.to_string());

        // if found.is_some() {
        //     let user = found.unwrap().user.read();
        //     return Some(user.clone());
        // } else {
        //     return Some(msg.author.clone());
        // }

        Ok(None)
    }
}

impl Iterator for MessageContext {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.args.len() > 1 {
            let mut args = self.args.clone().into_iter();
            let arg = args.next();
            self.args = args.collect::<Vec<Self::Item>>();
            arg
        } else {
            None
        }
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
