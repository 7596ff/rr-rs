use std::str;

use anyhow::Result;
use chrono::{Timelike, Utc};
use futures_util::future;
use log::{error, info};
use rand::seq::SliceRandom;
use serde::Deserialize;
use twilight_model::id::{GuildId, MessageId};

use crate::{
    model::Context,
    table::{
        raw::{RawImage, RawSetting},
        Image, Setting,
    },
};

#[derive(Debug, Deserialize)]
struct RawPartialImage {
    guild_id: String,
    message_id: String,
}

#[derive(Debug)]
struct PartialImage {
    guild_id: GuildId,
    message_id: MessageId,
}

impl From<RawPartialImage> for PartialImage {
    fn from(other: RawPartialImage) -> Self {
        Self {
            guild_id: GuildId(other.guild_id.parse::<u64>().unwrap()),
            message_id: MessageId(other.message_id.parse::<u64>().unwrap()),
        }
    }
}

async fn rotate_guild(context: Context, images: &[PartialImage], guild_id: GuildId) -> Result<()> {
    info!("rotating guild {}", guild_id);
    let now = Utc::now();

    let guild_id_string = guild_id.to_string();

    // first, determine if the guild icon should change.
    let setting: Setting = {
        let row = context
            .postgres
            .query_one(
                "SELECT * FROM settings WHERE
                (guild_id = $1);",
                &[&guild_id_string],
            )
            .await?;

        let raw: RawSetting = serde_postgres::from_row(&row)?;
        Setting::from(raw)
    };

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
    let mut redis = context.redis.get().await;
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
    let full_image: Image = {
        let row = context
            .postgres
            .query_one(
                "SELECT * FROM images WHERE
                (message_id = $1);",
                &[&chosen_image.unwrap().message_id.to_string()],
            )
            .await?;

        let raw: RawImage = serde_postgres::from_row(&row)?;
        Image::from(raw)
    };

    // and change the icon
    context
        .http
        .update_guild(guild_id)
        .icon(format!(
            "data:image/png;base64,{}",
            base64::encode(full_image.image)
        ))
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

pub async fn execute(context: Context) -> Result<()> {
    // get the data required for unique images
    let images: Vec<PartialImage> = {
        let rows = context
            .postgres
            .query("SELECT guild_id, message_id FROM images;", &[])
            .await?;
        let raw: Vec<RawPartialImage> = serde_postgres::from_rows(&rows)?;
        raw.into_iter().map(PartialImage::from).collect()
    };

    // build a new vec of unique guild ids
    let mut guild_ids: Vec<GuildId> = Vec::new();
    for image in images.iter() {
        if !guild_ids.contains(&image.guild_id) {
            guild_ids.push(image.guild_id);
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
