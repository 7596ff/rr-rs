use std::fmt::{Display, Formatter, Result as FmtResult};

use anyhow::Result;
use twilight::{
    http::error::Result as HttpResult,
    model::{
        channel::{Message, ReactionType},
        gateway::payload::ReactionAdd,
        user::User,
    },
};

use crate::model::{MessageContext, Response};

#[derive(Debug)]
enum FindMemberError {
    NoGuild,
}

impl Display for FindMemberError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::NoGuild => f.write_str("no guild found"),
        }
    }
}

impl std::error::Error for FindMemberError {}

pub async fn find_member(context: &MessageContext, _search_str: &str) -> Result<Option<User>> {
    if !context.message.mentions.is_empty() {
        let user = context.message.mentions.values().next().unwrap();
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

pub fn construct_response(sent: HttpResult<Message>) -> Response {
    match sent {
        Ok(msg) => Response::Some(msg),
        Err(why) => Response::Err(why),
    }
}

pub async fn did_you_mean(context: &MessageContext, name: &String) -> Result<bool> {
    let bystander = context
        .http
        .create_message(context.message.channel_id)
        .content(format!("Did you mean: \"{}\"?", name))?
        .await?;

    let emojis = [
        ReactionType::Unicode {
            name: "✅".to_string(),
        },
        ReactionType::Unicode {
            name: "❎".to_string(),
        },
    ];

    for emoji in &emojis {
        context
            .http
            .create_reaction(context.message.channel_id, bystander.id, emoji.clone())
            .await?;
    }

    let author_id = context.message.author.id;
    let reaction = context
        .standby
        .wait_for_reaction(bystander.id, move |event: &ReactionAdd| {
            event.user_id == author_id
        })
        .await?;

    context
        .http
        .delete_message(bystander.channel_id, bystander.id)
        .await?;

    Ok(reaction.emoji == emojis[0])
}
