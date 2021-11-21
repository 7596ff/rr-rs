use darkredis::ConnectionPool as RedisPool;
use hyper::client::{Client as HyperClient, HttpConnector};
use hyper_rustls::HttpsConnector;
use sqlx::PgPool;
use std::{error::Error, sync::Arc};
use twilight_cache_inmemory::InMemoryCache;
use twilight_http::{request::channel::reaction::RequestReactionType, Client as HttpClient};
use twilight_model::{
    channel::{message::Mention, Message, ReactionType},
    gateway::payload::incoming::{MessageCreate, ReactionAdd},
    id::EmojiId,
};
use twilight_standby::Standby;

pub type GenericError = Box<dyn Error + Send + Sync>;

pub enum ResponseReaction {
    Success,
    Failure,
}

impl ResponseReaction {
    pub fn id(&self) -> EmojiId {
        match *self {
            Self::Success => EmojiId::new(726252875696570368).expect("non zero"),
            Self::Failure => EmojiId::new(726253240806670367).expect("non zero"),
        }
    }

    pub fn value(&self) -> RequestReactionType {
        match *self {
            Self::Success => RequestReactionType::Custom {
                id: self.id(),
                name: Some("yeah"),
            },
            Self::Failure => RequestReactionType::Custom {
                id: self.id(),
                name: Some("nah"),
            },
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
pub struct BaseContext(Arc<BaseContextRef>);

impl BaseContext {
    pub fn new(
        cache: InMemoryCache,
        http: HttpClient,
        hyper: HyperClient<HttpsConnector<HttpConnector>>,
        postgres: PgPool,
        redis: RedisPool,
        standby: Standby,
    ) -> Self {
        Self(Arc::new(BaseContextRef {
            cache,
            http,
            hyper,
            postgres,
            redis,
            standby,
        }))
    }

    pub fn cache(&self) -> &InMemoryCache {
        &self.0.cache
    }

    pub fn http(&self) -> &HttpClient {
        &self.0.http
    }

    pub fn hyper(&self) -> &HyperClient<HttpsConnector<HttpConnector>> {
        &self.0.hyper
    }

    pub fn postgres(&self) -> &PgPool {
        &self.0.postgres
    }

    pub fn redis(&self) -> &RedisPool {
        &self.0.redis
    }

    pub fn standby(&self) -> &Standby {
        &self.0.standby
    }
}

#[derive(Debug)]
pub struct BaseContextRef {
    pub cache: InMemoryCache,
    pub http: HttpClient,
    pub hyper: HyperClient<HttpsConnector<HttpConnector>>,
    pub postgres: PgPool,
    pub redis: RedisPool,
    pub standby: Standby,
}

#[derive(Clone, Debug)]
pub struct MessageContext {
    base: BaseContext,
    pub message: Box<MessageCreate>,
    pub args: Vec<String>,
}

impl MessageContext {
    pub fn new(base: BaseContext, message: Box<MessageCreate>) -> Result<Self, GenericError> {
        let args = shellwords::split(&message.content)?;

        Ok(Self {
            base,
            message,
            args,
        })
    }

    pub fn cache(&self) -> &InMemoryCache {
        self.base.cache()
    }

    pub async fn confirm(&self, content: impl Into<String>) -> Result<bool, GenericError> {
        // make a bystander message
        let content = content.into();
        let bystander = self
            .http()
            .create_message(self.message.channel_id)
            .content(&content)?
            .exec()
            .await?
            .model()
            .await?;

        // react with check and x
        self.http()
            .create_reaction(
                self.message.channel_id,
                bystander.id,
                &ResponseReaction::Success.value(),
            )
            .exec()
            .await?;

        self.http()
            .create_reaction(
                self.message.channel_id,
                bystander.id,
                &ResponseReaction::Failure.value(),
            )
            .exec()
            .await?;

        // wait for the user to respond to the menu
        let author_id = self.message.author.id;
        let reaction = self
            .standby()
            .wait_for_reaction(bystander.id, move |event: &ReactionAdd| {
                event.user_id == author_id
            })
            .await?;

        // clear out the message and return the result
        self.http()
            .delete_message(bystander.channel_id, bystander.id)
            .exec()
            .await?;

        Ok(if let ReactionType::Custom { id, .. } = reaction.emoji {
            id == ResponseReaction::Success.id()
        } else {
            false
        })
    }

    pub async fn find_member(&self) -> Result<Option<Mention>, GenericError> {
        if !self.message.mentions.is_empty() {
            return Ok(Some(self.message.mentions[0].clone()));
        }

        // TODO: wait for CachedGuild.members
        //
        // let guild_id = context.message.guild_id.ok_or(FindMemberError::NoGuild)?;
        // let guild = context.base.0.cache.guild(guild_id).await?;

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

    pub fn http(&self) -> &HttpClient {
        self.base.http()
    }

    pub fn hyper(&self) -> &HyperClient<HttpsConnector<HttpConnector>> {
        self.base.hyper()
    }

    pub fn postgres(&self) -> &PgPool {
        self.base.postgres()
    }

    pub async fn react(&self, emoji: &RequestReactionType<'_>) -> Result<(), GenericError> {
        self.http()
            .create_reaction(self.message.channel_id, self.message.id, emoji)
            .exec()
            .await?;

        Ok(())
    }

    pub fn redis(&self) -> &RedisPool {
        self.base.redis()
    }

    pub async fn reply(&self, content: impl Into<String>) -> Result<Message, GenericError> {
        let message = self
            .http()
            .create_message(self.message.channel_id)
            .content(&content.into())
            .unwrap()
            .exec()
            .await?
            .model()
            .await?;

        Ok(message)
    }

    pub fn standby(&self) -> &Standby {
        self.base.standby()
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
