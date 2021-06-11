use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    sync::Arc,
};

use anyhow::{Error as Anyhow, Result};
use darkredis::ConnectionPool as RedisPool;
use serde::de::Deserialize;
use serde_postgres::Deserializer;
use tokio_postgres::{types::ToSql, Client as PgClient};
use twilight_cache_inmemory::InMemoryCache;
use twilight_http::{
    error::Error as HttpError, request::channel::reaction::RequestReactionType,
    Client as HttpClient,
};
use twilight_model::{
    channel::{message::Mention, Message},
    gateway::payload::{MessageCreate, ReactionAdd},
    id::EmojiId,
};
use twilight_standby::Standby;

pub enum ResponseReaction {
    Success,
    Failure,
}

impl ResponseReaction {
    pub fn value(&self) -> RequestReactionType {
        match *self {
            Self::Success => RequestReactionType::Custom {
                id: EmojiId(726252875696570368),
                name: Some("yeah".into()),
            },
            Self::Failure => RequestReactionType::Custom {
                id: EmojiId(726253240806670367),
                name: Some("nah".into()),
            },
        }
    }
}

#[derive(Debug)]
pub enum SettingRole {
    Movies,
}

impl Display for SettingRole {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Self::Movies => write!(f, "movies"),
        }
    }
}

#[derive(Debug)]
pub enum Response {
    Message(Message),
    Reaction,
    None,
}

#[derive(Clone, Debug)]
pub struct Context {
    pub cache: InMemoryCache,
    pub http: HttpClient,
    pub postgres: Arc<PgClient>,
    pub redis: RedisPool,
    pub standby: Standby,
}

impl<'de> Context {
    pub async fn query_one<R: Deserialize<'de>>(
        &self,
        statement: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<R> {
        let row = self.postgres.query_one(statement, params).await?;

        let mut deserializer = Deserializer::from_row(&row);

        R::deserialize(&mut deserializer).map_err(Anyhow::new)
    }
}

#[derive(Clone, Debug)]
pub struct MessageContext {
    pub cache: InMemoryCache,
    pub http: HttpClient,
    pub postgres: Arc<PgClient>,
    pub redis: RedisPool,
    pub standby: Standby,
    pub message: Box<MessageCreate>,
    pub args: Vec<String>,
}

impl MessageContext {
    pub fn new(context: Context, message: Box<MessageCreate>) -> Result<Self> {
        let args = shellwords::split(&message.content)?;

        Ok(Self {
            cache: context.cache,
            http: context.http,
            postgres: context.postgres,
            redis: context.redis,
            standby: context.standby,
            message,
            args,
        })
    }

    pub async fn reply(&self, content: impl Into<String>) -> Result<Message, HttpError> {
        self.http
            .create_message(self.message.channel_id)
            .content(content.into())
            .unwrap()
            .await
    }

    pub async fn react(&self, emoji: RequestReactionType) -> Result<(), HttpError> {
        self.http
            .create_reaction(self.message.channel_id, self.message.id, emoji)
            .await
    }

    pub async fn confirm(&self, content: impl Into<String>) -> Result<bool> {
        // make a bystander message
        let bystander = self
            .http
            .create_message(self.message.channel_id)
            .content(content.into())?
            .await?;

        // react with check and x
        self.http
            .create_reaction(
                self.message.channel_id,
                bystander.id,
                ResponseReaction::Success.value(),
            )
            .await?;

        self.http
            .create_reaction(
                self.message.channel_id,
                bystander.id,
                ResponseReaction::Failure.value(),
            )
            .await?;

        // wait for the user to respond to the menu
        let author_id = self.message.author.id;
        let reaction = self
            .standby
            .wait_for_reaction(bystander.id, move |event: &ReactionAdd| {
                event.user_id == author_id
            })
            .await?;

        // clear out the message and return the result
        self.http
            .delete_message(bystander.channel_id, bystander.id)
            .await?;
        Ok(RequestReactionType::from(reaction.emoji.to_owned())
            == ResponseReaction::Success.value())
    }

    pub async fn find_member(&self) -> Result<Option<Mention>> {
        if !self.message.mentions.is_empty() {
            return Ok(Some(self.message.mentions[0].clone()));
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

impl<'de> MessageContext {
    pub async fn query<R: Deserialize<'de>>(
        &self,
        statement: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Vec<R>> {
        let rows = self.postgres.query(statement, params).await?;

        rows.iter()
            .map(|row| Deserializer::from_row(row))
            .map(|mut row| R::deserialize(&mut row))
            .collect::<Result<Vec<R>, _>>()
            .map_err(Anyhow::new)
    }

    pub async fn query_one<R: Deserialize<'de>>(
        &self,
        statement: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<R> {
        let row = self.postgres.query_one(statement, params).await?;

        let mut deserializer = Deserializer::from_row(&row);

        R::deserialize(&mut deserializer).map_err(Anyhow::new)
    }

    pub async fn query_opt<R: Deserialize<'de>>(
        &self,
        statement: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Option<R>> {
        let row = self.postgres.query_opt(statement, params).await?;

        if let Some(row) = row {
            let mut deserializer = Deserializer::from_row(&row);

            let row = R::deserialize(&mut deserializer)?;

            Ok(Some(row))
        } else {
            Ok(None)
        }
    }
}

impl Iterator for MessageContext {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.args.is_empty() {
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
    pub postgres: Arc<PgClient>,
    pub redis: RedisPool,
    pub reaction: Box<ReactionAdd>,
}

impl ReactionContext {
    pub fn new(context: Context, reaction: Box<ReactionAdd>) -> Self {
        Self {
            cache: context.cache,
            http: context.http,
            postgres: context.postgres,
            redis: context.redis,
            reaction,
        }
    }
}
