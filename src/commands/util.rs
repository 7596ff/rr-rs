use crate::{
    model::{GenericError, MessageContext, Response, ResponseReaction},
    table::Emoji,
};
use anyhow::anyhow;
use chrono::{Duration, Utc};
use http::uri::Uri;
use hyper::{
    body::{self, Body},
    Request,
};
use lazy_static::lazy_static;
use rand::seq::SliceRandom;
use regex::Regex;
use std::{collections::HashMap, convert::TryFrom, fmt::Write};
use twilight_http::request::channel::reaction::RequestReactionType;

const HELP_TEXT: &str = include_str!("../../help.txt");

lazy_static! {
    static ref E: Regex =
        Regex::new(r"<a?:(?P<name>[a-zA-Z1-9-_]{2,}):(?P<id>\d{17,21})>").unwrap();
}

pub async fn avatar(context: &mut MessageContext) -> Result<Response, GenericError> {
    let found_user = context.find_member().await?;

    let (id, avatar) = match found_user {
        Some(user) => (user.id, user.avatar),
        None => (
            context.message.author.id,
            context.message.author.avatar.clone(),
        ),
    };

    if let Some(avatar) = avatar {
        let content = format!(
            "https://cdn.discordapp.com/avatars/{}/{}?size=2048",
            id, avatar
        );

        let reply = context.reply(content).await?;
        return Ok(Response::Message(reply));
    }

    Ok(Response::None)
}

pub async fn choose(context: &MessageContext) -> Result<Response, GenericError> {
    let maybe_item = context.args.choose(&mut rand::thread_rng());

    let item = match maybe_item {
        Some(item) => item,
        None => return Ok(Response::None),
    };

    let reply = context.reply(item).await?;

    Ok(Response::Message(reply))
}

pub async fn emojis(context: &MessageContext) -> Result<Response, GenericError> {
    let one_week_ago = Utc::now()
        .checked_sub_signed(Duration::days(7))
        .unwrap()
        .timestamp();

    let emojis = sqlx::query_as!(
        Emoji,
        "SELECT
            datetime,
            guild_id AS \"guild_id: _\",
            message_id AS \"message_id: _\",
            member_id AS \"member_id: _\",
            emoji_id AS \"emoji_id: _\",
            reaction
        FROM emojis WHERE
        (datetime >= $1 AND guild_id = $2);",
        one_week_ago,
        context.message.guild_id.unwrap().to_string(),
    )
    .fetch_all(context.postgres())
    .await?;

    let mut counts = emojis
        .iter()
        .fold(HashMap::new(), |mut counts, emoji| {
            {
                let counter = counts.entry(emoji.emoji_id.to_string()).or_insert(0);
                *counter += 1;
            }

            counts
        })
        .into_iter()
        .collect::<Vec<(String, i32)>>();

    counts.sort_by(|a, b| b.0.cmp(&a.0));
    counts.sort_by(|a, b| b.1.cmp(&a.1));

    let content = counts
        .iter()
        .enumerate()
        .map(|(index, (id, count))| {
            let mut formatted = format!("`{}` <:deleted:{}> ", count, id);
            if (index + 1) % 10 == 0 {
                formatted = format!("{}\n", formatted);
            }

            formatted
        })
        .collect::<Vec<String>>()
        .join("");

    let reply = context.reply(content).await?;

    Ok(Response::Message(reply))
}

pub async fn help(context: &MessageContext) -> Result<Response, GenericError> {
    let reply = context.reply(HELP_TEXT).await?;

    Ok(Response::Message(reply))
}

pub async fn invite(context: &MessageContext) -> Result<Response, GenericError> {
    let content = "<https://discordapp.com/oauth2/authorize?client_id=254387001556598785&permissions=268435488&scope=bot>";

    let reply = context.reply(content).await?;

    Ok(Response::Message(reply))
}

pub async fn ping(context: &MessageContext) -> Result<Response, GenericError> {
    let sent = context.reply("pong!").await?;

    let sent_time = sent.timestamp.as_micros();
    let message_time = context.message.timestamp.as_micros();
    let latency_in_milliseconds = (sent_time - message_time) / 1000;

    let new_content = format!("🏓 Message send latency: {} ms", latency_in_milliseconds);

    let update = context
        .http()
        .update_message(context.message.channel_id, sent.id)
        .content(Some(&new_content))?
        .exec()
        .await?
        .model()
        .await?;

    Ok(Response::Message(update))
}

pub async fn shuffle(context: &mut MessageContext) -> Result<Response, GenericError> {
    let mut args = context.args.clone();
    args.shuffle(&mut rand::thread_rng());

    let mut content = String::new();
    for (counter, item) in context.args.iter().enumerate() {
        writeln!(content, "`{}` {}", counter, item)?;
    }

    let reply = context.reply(content).await?;

    Ok(Response::Message(reply))
}

pub async fn steal(context: &mut MessageContext) -> Result<Response, GenericError> {
    if let Some(emoji) = context.next() {
        // create variables that hold the chain of information priority
        let mut uri: Option<Uri> = Uri::try_from(&emoji).ok();
        let mut name: Option<String> = None;

        // set the uri and name from a custom emoji match
        if uri.is_none() {
            let caps = E.captures(&emoji).ok_or_else(|| anyhow!("no match."))?;
            let formatted = format!(
                "https://cdn.discordapp.com/emojis/{}.png?v=1",
                caps.name("id").ok_or_else(|| anyhow!("no id"))?.as_str()
            );

            uri = Uri::try_from(formatted).ok();
            name = caps.name("name").map(|m| String::from(m.as_str()));
        }

        // override the name if there is another argument
        if let Some(arg_name) = context.next() {
            name = Some(arg_name);
        }

        // default the name to emoji if none
        if name.is_none() {
            name = Some("emoji".into());
        }

        // upload the emoji if everything checks out
        if let (Some(uri), Some(name)) = (uri, name) {
            let request = Request::get(uri).body(Body::empty())?;
            let mut response = context.hyper().request(request).await?;
            let buffer = body::to_bytes(response.body_mut()).await?;

            let image = format!("data:image/png;base64,{}", base64::encode(buffer));

            let emoji = context
                .http()
                .create_emoji(context.message.guild_id.unwrap(), &name, image.as_str())
                .exec()
                .await?
                .model()
                .await?;

            context.react(&ResponseReaction::Success.value()).await?;
            context
                .react(&RequestReactionType::Custom {
                    id: emoji.id,
                    name: Some(name.as_str()),
                })
                .await?;

            return Ok(Response::Reaction);
        }
    }

    let reply = context.reply("USAGE: katze steal <emoji> [<name>]").await?;

    Ok(Response::Message(reply))
}
