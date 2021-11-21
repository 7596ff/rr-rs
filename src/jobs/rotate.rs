use crate::{
    model::BaseContext,
    table::{
        id::{SqlxGuildId, SqlxMessageId},
        Image, Setting,
    },
};
use anyhow::Result;
use chrono::{Timelike, Utc};
use futures_util::future;
use log::{error, info};
use rand::seq::SliceRandom;
use std::str;
use twilight_model::id::GuildId;

#[derive(Debug)]
struct PartialImage {
    guild_id: SqlxGuildId,
    message_id: SqlxMessageId,
}

async fn rotate_guild(
    context: BaseContext,
    images: &[PartialImage],
    guild_id: GuildId,
) -> Result<()> {
    info!("rotating guild {}", guild_id);
    let now = Utc::now();

    let guild_id_string = guild_id.to_string();

    // first, determine if the guild icon should change.
    let setting = Setting::query(context.postgres().clone(), guild_id).await?;

    // don't rotate if we shouldn't
    if !setting.rotate_enabled {
        return Ok(());
    }

    // mod the current hour, one-indexed, by the guild's rotate_every setting
    // if it's a multiple, we rotate; if not, return
    if (now.hour() + 1) as i32 % setting.rotate_every != 0 {
        return Ok(());
    }

    // compare the last rotation time plus the offset to now
    let mut redis = context.redis().get().await;
    let last_time = redis.hget("rr-rs:rotations", &guild_id_string).await?;

    // if there's no response use 0 as the time
    let last_time = match last_time {
        Some(last_time) => str::from_utf8(&last_time)?.parse::<i32>()?,
        None => 0,
    };

    if (last_time + setting.rotate_every * 60) as i64 > now.timestamp() {
        return Ok(());
    }

    // filter the guild's images
    let filtered_images: Vec<&PartialImage> = images
        .iter()
        .filter(|image| image.guild_id == guild_id)
        .collect();

    // randomly choosing one, if it exists
    let chosen_image = filtered_images.choose(&mut rand::thread_rng());
    if chosen_image.is_none() {
        return Ok(());
    }

    // get the image data
    let full_image = sqlx::query_as!(
        Image,
        "SELECT
            guild_id AS \"guild_id: _\",
            message_id AS \"message_id: _\",
            image,
            filetype
        FROM images WHERE
        (message_id = $1);",
        chosen_image.unwrap().message_id.to_string(),
    )
    .fetch_one(context.postgres())
    .await?;

    // and change the icon
    let icon = format!("data:image/png;base64,{}", base64::encode(full_image.image));
    context
        .http()
        .update_guild(guild_id)
        .icon(Some(&icon))
        .exec()
        .await?;

    // tell redis the last time we rotated
    redis
        .hset(
            "rr-rs:rotations",
            &guild_id_string,
            now.timestamp().to_string(),
        )
        .await?;

    Ok(())
}

pub async fn execute(context: BaseContext) -> Result<()> {
    // get the data required for unique images
    let images = sqlx::query_as!(
        PartialImage,
        "SELECT
            guild_id AS \"guild_id: _\",
            message_id AS \"message_id: _\"
        FROM images;",
    )
    .fetch_all(context.postgres())
    .await?;

    // build a new vec of unique guild ids
    let mut guild_ids: Vec<GuildId> = Vec::new();
    for image in images.iter() {
        if !guild_ids.contains(&image.guild_id.0) {
            guild_ids.push(image.guild_id.0);
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
