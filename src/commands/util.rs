use std::{collections::HashMap, convert::TryFrom, fmt::Write};

use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Utc};
use futures_util::io::AsyncReadExt;
use http::uri::Uri;
use lazy_static::lazy_static;
use rand::seq::SliceRandom;
use regex::Regex;
use twilight_http::request::channel::reaction::RequestReactionType;

use crate::{
    model::{MessageContext, Response, ResponseReaction},
    table::{raw::RawEmoji, Emoji},
};

const HELP_TEXT: &str = include_str!("../../help.txt");

lazy_static! {
    static ref E: Regex =
        Regex::new(r"<a?:(?P<name>[a-zA-Z1-9-_]{2,}):(?P<id>\d{17,21})>").unwrap();
}

pub async fn avatar(context: &mut MessageContext) -> Result<Response> {
    let found_user = context.find_member().await?;

    let user = match found_user {
        Some(user) => user,
        None => context.message.author.clone(),
    };

    if let Some(avatar) = user.avatar {
        let content =
            format!("https://cdn.discordapp.com/avatars/{}/{}?size=2048", user.id, avatar);

        let reply = context.reply(content).await?;
        return Ok(Response::Message(reply));
    }

    Ok(Response::None)
}

pub async fn choose(context: &MessageContext) -> Result<Response> {
    let maybe_item = context.args.choose(&mut rand::thread_rng());

    let item = match maybe_item {
        Some(item) => item,
        None => return Ok(Response::None),
    };

    let reply = context.reply(item).await?;
    Ok(Response::Message(reply))
}

pub async fn emojis(context: &MessageContext) -> Result<Response> {
    let one_week_ago = Utc::now().checked_sub_signed(Duration::days(7)).unwrap().timestamp();

    let emojis: Vec<Emoji> = {
        let rows = context
            .postgres
            .query(
                "SELECT * FROM emojis WHERE
                (datetime >= $1 AND guild_id = $2);",
                &[&one_week_ago, &context.message.guild_id.unwrap().to_string()],
            )
            .await?;

        let raw: Vec<RawEmoji> = serde_postgres::from_rows(&rows)?;
        raw.into_iter().map(Emoji::from).collect()
    };

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

pub async fn help(context: &MessageContext) -> Result<Response> {
    let reply = context.reply(HELP_TEXT).await?;
    Ok(Response::Message(reply))
}

pub async fn invite(context: &MessageContext) -> Result<Response> {
    let content = "<https://discordapp.com/oauth2/authorize?client_id=254387001556598785&permissions=268435488&scope=bot>";

    let reply = context.reply(content).await?;
    Ok(Response::Message(reply))
}

pub async fn ping(context: &MessageContext) -> Result<Response> {
    let sent = context.reply("pong!").await?;

    let sent_time = DateTime::parse_from_rfc3339(sent.timestamp.as_str())?;
    let message_time = DateTime::parse_from_rfc3339(context.message.timestamp.as_str())?;
    let latency = sent_time.timestamp_millis() - message_time.timestamp_millis();

    let new_content = format!("🏓 Message send latency: {} ms", latency);
    let update = context
        .http
        .update_message(context.message.channel_id, sent.id)
        .content(new_content)?
        .await?;

    Ok(Response::Message(update))
}

pub async fn shuffle(context: &mut MessageContext) -> Result<Response> {
    context.args.shuffle(&mut rand::thread_rng());
    let mut content = String::new();

    for (counter, item) in context.args.iter().enumerate() {
        writeln!(content, "`{}` {}", counter, item)?;
    }

    let reply = context.reply(content).await?;
    Ok(Response::Message(reply))
}

pub async fn steal(context: &mut MessageContext) -> Result<Response> {
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
        if uri.is_some() && name.is_some() {
            let mut resp = isahc::get_async(uri.unwrap()).await?;
            let mut buffer: Vec<u8> = Vec::new();
            resp.body_mut().read_to_end(&mut buffer).await?;

            let emoji = context
                .http
                .create_emoji(
                    context.message.guild_id.unwrap(),
                    name.unwrap(),
                    format!("data:image/png;base64,{}", base64::encode(buffer)),
                )
                .await?;

            context.react(ResponseReaction::Success.value()).await?;
            context
                .react(RequestReactionType::Custom { id: emoji.id, name: Some(emoji.name) })
                .await?;
            return Ok(Response::Reaction);
        }
    }

    let reply = context.reply("USAGE: katze steal <emoji> [<name>]").await?;
    Ok(Response::Message(reply))
}
