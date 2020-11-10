use anyhow::Result;
use chrono::{DateTime, TimeZone, Utc};
use chrono_humanize::{Accuracy, HumanTime, Tense};
use futures_util::future;
use lazy_static::lazy_static;
use regex::Regex;
use twilight_mention::Mention;

use crate::{
    model::{MessageContext, Response},
    table::Setting,
};

lazy_static! {
    static ref E: Regex = Regex::new(r"<a?:[a-zA-Z1-9-_]{2,}:(?P<id>\d{17,21})>").unwrap();
    static ref V: Regex = Regex::new(r"\b[Vv][Oo][Rr][Ee]").unwrap();
}

const THIRTY_MINUTES: i64 = 1800000;

pub async fn emojis(context: &MessageContext) -> Result<Response> {
    let now = Utc::now();

    let ids = E
        .captures_iter(context.message.content.as_ref())
        .map(|c| c["id"].to_string())
        .collect::<Vec<String>>();

    let futures = ids.iter().map(|id| {
        sqlx::query!(
            "INSERT INTO emojis (datetime, guild_id, message_id, member_id, emoji_id)
            VALUES ($1, $2, $3, $4, $5);",
            now.timestamp(),
            context.message.guild_id.unwrap().to_string(),
            context.message.id.to_string(),
            context.message.author.id.to_string(),
            id
        )
        .execute(&context.pool)
    });

    future::join_all(futures).await;

    Ok(Response::None)
}

pub async fn vtrack(context: &MessageContext) -> Result<Response> {
    let guild_id = context.message.guild_id.unwrap().to_string();

    if V.is_match(context.message.content.as_ref()) {
        let setting = sqlx::query_as!(
            Setting,
            "SELECT * FROM settings WHERE
            (guild_id = $1);",
            &guild_id
        )
        .fetch_one(&context.pool)
        .await?;

        if setting.vtrack {
            // get the last time it was used, or 0 otherwise
            let mut redis = context.redis.get().await;
            let reply = redis
                .hget("katze:vore", &guild_id)
                .await?
                .unwrap_or_else(|| b"0".to_owned().to_vec());

            // determine the difference. use naive utc because we know both times are utc, and we
            // only care about the resultant duration
            let last_timestamp =
                Utc.timestamp(String::from_utf8(reply)?.parse::<i64>()?, 0).naive_utc();
            let message_timestamp =
                DateTime::parse_from_rfc3339(context.message.timestamp.as_ref())?.naive_utc();
            let difference = message_timestamp - last_timestamp;

            // set the latest time
            redis.hset("katze:vore", &guild_id, &message_timestamp.timestamp().to_string()).await?;

            // determine if we should send a message
            if difference.num_milliseconds() >= THIRTY_MINUTES {
                let content = format!(
                    "{} has broken the silence and said the cursed word.
This server has gone {} since the last infraction.",
                    context.message.author.mention(),
                    HumanTime::from(difference).to_text_en(Accuracy::Precise, Tense::Present),
                );

                let reply = context.reply(content).await?;
                return Ok(Response::Message(reply));
            }
        }
    }

    Ok(Response::None)
}
