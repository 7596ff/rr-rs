use std::str;

use anyhow::Result;
use chrono::Utc;
use rand::{seq::SliceRandom, thread_rng};

use crate::{
    model::{MessageContext, Response},
    table::{Image, Setting},
};

#[derive(Debug)]
struct PartialImage {
    message_id: String,
}

async fn rotate(context: &MessageContext) -> Result<Response> {
    let guild_id = context.message.guild_id.unwrap().to_string();
    let now = Utc::now();

    // get the last time this guild rotated
    let mut redis = context.redis.get().await;
    let last_time = redis.hget("rr-rs:rotations", &guild_id).await?;

    // if there's no response use 0 as the time
    let last_time = match last_time {
        Some(last_time) => str::from_utf8(&last_time)?.parse::<i32>()?,
        None => 0,
    };

    // check if it's only been ten minutes, rotate if it's been more
    if (last_time + 600) as i64 > now.timestamp() {
        let reply = context.reply("You are rotating too fast!").await?;
        return Ok(Response::Message(reply));
    }

    // get the guild settings
    let setting = sqlx::query_as!(
        Setting,
        "SELECT * FROM settings WHERE
        (guild_id = $1);",
        &guild_id
    )
    .fetch_one(&context.pool)
    .await?;

    // check if we should rotate
    if !setting.rotate_enabled {
        let reply = context.reply("Rotation is disabled for this server.").await?;
        return Ok(Response::Message(reply));
    }

    // get a list of partial images
    let partial_images = sqlx::query_as!(
        PartialImage,
        "SELECT message_id FROM images WHERE
        (guild_id = $1);",
        &guild_id
    )
    .fetch_all(&context.pool)
    .await?;

    // pick an image
    let partial_image = partial_images.choose(&mut thread_rng()).unwrap();

    // get the full image
    let full_image = sqlx::query_as!(
        Image,
        "SELECT * FROM images WHERE
        (message_id = $1);",
        partial_image.message_id
    )
    .fetch_one(&context.pool)
    .await?;

    // and change the icon
    context
        .http
        .update_guild(context.message.guild_id.unwrap())
        .icon(format!("data:image/png;base64,{}", base64::encode(full_image.image)))
        .await?;

    // tell redis the last time we rotated
    redis.hset("rr-rs:rotations", &guild_id, now.timestamp().to_string()).await?;

    Ok(Response::None)
}

pub async fn execute(context: &mut MessageContext) -> Result<Response> {
    rotate(&context).await
}
