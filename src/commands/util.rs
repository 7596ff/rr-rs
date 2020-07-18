use std::fmt::Write;

use anyhow::Result;
use chrono::DateTime;
use rand::{seq::SliceRandom, thread_rng};

use crate::model::{MessageContext, Response};

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
    let maybe_item = context.args.choose(&mut thread_rng());

    let item = match maybe_item {
        Some(item) => item,
        None => return Ok(Response::None),
    };

    let reply = context.reply(item).await?;
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

    let update = context
        .http
        .update_message(context.message.channel_id, sent.id)
        .content(format!("🏓 Message send latency: {} ms", latency))?
        .await?;

    Ok(Response::Message(update))
}

pub async fn shuffle(context: &mut MessageContext) -> Result<Response> {
    context.args.shuffle(&mut thread_rng());
    let mut content = String::new();

    for (counter, item) in context.args.iter().enumerate() {
        writeln!(content, "`{}` {}", counter, item)?;
    }

    let reply = context.reply(content).await?;
    Ok(Response::Message(reply))
}
