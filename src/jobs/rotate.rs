use std::str;

use anyhow::Result;
use chrono::{Timelike, Utc};
use futures_util::future;
use log::{error, info};
use rand::{seq::SliceRandom, thread_rng};
use twilight::model::id::GuildId;

use crate::{
    model::Context,
    table::{Image, Setting},
};

#[derive(Debug)]
struct PartialImage {
    guild_id: String,
    message_id: String,
}

async fn rotate_guild(context: Context, images: &[PartialImage], guild_id: String) -> Result<()> {
    info!("rotating guild {}", guild_id);
    let now = Utc::now();

    // first, determine if the guild icon should change.
    let setting = sqlx::query_as!(
        Setting,
        "SELECT * FROM settings WHERE
        (guild_id = $1);",
        guild_id
    )
    .fetch_one(&context.pool)
    .await?;

    // mod the current hour, one-indexed, by the guild's rotate_every setting
    // if it's not a multiple, continue
    if (now.hour() + 1) as i32 % setting.rotate_every != 0 {
        return Ok(());
    }

    // compare the last rotation time plus the offset to now
    let mut redis = context.redis.get().await;
    let last_time = redis.hget("rotations", &guild_id).await?;

    // if there's no response use 0 as the time
    let last_time = match last_time {
        Some(last_time) => str::from_utf8(&last_time)?.parse::<i32>()?,
        None => 0,
    };

    if (last_time + setting.rotate_every * 60) as i64 > now.timestamp() {
        return Ok(());
    }

    // filter the guild's images
    let filtered_images: Vec<&PartialImage> =
        images.iter().filter(|image| image.guild_id == guild_id).collect();

    // randomly choosing one, if it exists
    let chosen_image = filtered_images.choose(&mut thread_rng());
    if chosen_image.is_none() {
        return Ok(());
    }

    // get the image data
    let full_image = sqlx::query_as!(
        Image,
        "SELECT * FROM images WHERE
        (message_id = $1);",
        chosen_image.unwrap().message_id,
    )
    .fetch_one(&context.pool)
    .await?;

    // and change the icon
    context
        .http
        .update_guild(GuildId(guild_id.parse::<u64>()?))
        .icon(format!("data:image/png;base64,{}", base64::encode(full_image.image)))
        .await?;

    Ok(())
}

pub async fn execute(context: Context) -> Result<()> {
    // get the data required for unique images
    let images = sqlx::query_as!(PartialImage, "SELECT guild_id, message_id FROM images;")
        .fetch_all(&context.pool)
        .await?;

    // build a new vec of unique guild ids
    let mut guild_ids: Vec<String> = Vec::new();
    for image in images.iter() {
        if !guild_ids.contains(&image.guild_id) {
            guild_ids.push(image.guild_id.clone());
        }
    }

    // loop through the guild ids
    let mut tasks = Vec::new();
    for guild_id in guild_ids {
        tasks.push(rotate_guild(context.clone(), &images, guild_id));
    }
    let finished_tasks = future::join_all(tasks).await;

    // report any errors
    for task in finished_tasks {
        if let Err(why) = task {
            error!("rotation task failed\n{:?}", why);
        }
    }

    Ok(())
}