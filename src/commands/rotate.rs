use std::{
    fmt::{Display, Formatter, Result as FmtResult, Write},
    str,
};

use anyhow::Result;
use chrono::Utc;
use futures_util::io::AsyncReadExt;
use image::{imageops, jpeg::JPEGEncoder, ColorType, RgbImage};
use rand::seq::SliceRandom;
use twilight::model::{channel::Message, guild::Permissions};

use crate::{
    checks,
    model::{MessageContext, Response, ResponseReaction},
    table::{Image, Setting},
};

#[derive(Debug)]
enum ResizeError {
    DowncastFailure,
}

impl Display for ResizeError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Self::DowncastFailure => write!(f, "downcast failed"),
        }
    }
}

impl std::error::Error for ResizeError {}

#[derive(Debug)]
struct PartialImage {
    message_id: String,
}

pub async fn add_image(context: &MessageContext) -> Result<Response> {
    checks::has_permission(&context, Permissions::MANAGE_GUILD).await?;

    // use the first attachment, or whatever's left in the args
    let url = if context.message.attachments.is_empty() {
        context.args.join(" ")
    } else {
        context.message.attachments.first().unwrap().url.clone()
    };

    // download the image
    let mut resp = isahc::get_async(url).await?;
    let mut buffer: Vec<u8> = Vec::new();
    resp.body_mut().read_to_end(&mut buffer).await?;

    // guess the image format
    let format = image::guess_format(buffer.as_ref())?.extensions_str();

    // save the image to the database, as a raw response
    sqlx::query!(
        "INSERT INTO images (guild_id, message_id, image, filetype)
        VALUES ($1, $2, $3, $4);",
        context.message.guild_id.unwrap().to_string(),
        context.message.id.to_string(),
        buffer,
        format[0],
    )
    .execute(&context.pool)
    .await?;

    context.react(ResponseReaction::Success.value()).await?;
    Ok(Response::Reaction)
}

pub async fn count(context: &MessageContext) -> Result<Response> {
    let images = sqlx::query!(
        "SELECT COUNT(message_id) FROM images WHERE
        (guild_id = $1);",
        context.message.guild_id.unwrap().to_string()
    )
    .fetch_one(&context.pool)
    .await?;

    if let Some(count) = images.count {
        let reply = context.reply(format!("This server has **{}** images.", count)).await?;
        return Ok(Response::Message(reply));
    }

    Ok(Response::None)
}

pub async fn delete(context: &mut MessageContext) -> Result<Response> {
    if let Some(message_id) = context.next() {
        let image = sqlx::query_as!(
            Image,
            "DELETE FROM images WHERE
            (message_id = $1)
            RETURNING *;",
            message_id
        )
        .fetch_optional(&context.pool)
        .await?;

        if let Some(image) = image {
            let reply = context
                .http
                .create_message(context.message.channel_id)
                .content(format!("Deleted `{}`.", image.message_id))?
                .attachment(format!("{}.{}", image.message_id, image.filetype), image.image)
                .await?;

            Ok(Response::Message(reply))
        } else {
            let reply = context.reply(format!("Image `{}` not found.", message_id)).await?;
            Ok(Response::Message(reply))
        }
    } else {
        let reply = context.reply("No image specified.").await?;
        Ok(Response::Message(reply))
    }
}

pub async fn list(context: &MessageContext) -> Result<Response> {
    let mut images = sqlx::query_as!(
        Image,
        "SELECT * FROM images WHERE
        (guild_id = $1);",
        context.message.guild_id.unwrap().to_string()
    )
    .fetch_all(&context.pool)
    .await?;

    if images.len() > 18
        && !context
            .confirm(format!(
                "This server has {} images, that's a lot! Are you sure you want to list them all?",
                images.len()
            ))
            .await?
    {
        return Ok(Response::None);
    }

    context.http.create_typing_trigger(context.message.channel_id).await?;

    let mut reply: Option<Message> = None;
    while !images.is_empty() {
        // determine how many images are in this larger image
        let count = if images.len() < 18 { images.len() } else { 18 };

        // drain the images into a chunk of at most 18 images
        let chunk: Vec<Image> = images.drain(0..count).collect();
        let ids: Vec<&String> = chunk.iter().map(|image| &image.message_id).collect();

        // create a main image
        let mut main = RgbImage::new(100, 100);
        for (index, image) in chunk.iter().enumerate() {
            // load and resize the image
            let buffer = image::load_from_memory(&image.image)?;
            let buffer = buffer.resize(100, 100, imageops::Triangle);

            // downcast resized to rgb8
            let buffer = buffer.as_rgb8().ok_or(ResizeError::DowncastFailure)?;

            // determine coordinates of new image
            let (x, y) = ((index % 6) as u32, (index / 3) as u32);

            // create a new main image that is larger
            let mut new_main = RgbImage::new((x + 1) * 100, (y + 1) * 100);

            // overlay the old main image on the new one
            imageops::overlay(&mut new_main, &main, 0, 0);

            // add the new image at coordinates
            imageops::overlay(&mut new_main, buffer, x * 100, y * 100);

            // replace the main image
            main = new_main;
        }

        // encode the image as a jpeg with quality 95
        let mut encoded: Vec<u8> = Vec::new();
        let mut encoder = JPEGEncoder::new_with_quality(&mut encoded, 95u8);
        let (x, y) = main.dimensions();
        encoder.encode(&main.into_vec(), x, y, ColorType::Rgb8)?;

        let mut ids_fmt = String::new();
        for (index, id) in ids.iter().enumerate() {
            if (index + 1) % 6 == 0 {
                writeln!(ids_fmt, "`{}`", id)?;
            } else {
                write!(ids_fmt, "`{}` ", id)?;
            }
        }

        // send the image to discord
        reply = Some(
            context
                .http
                .create_message(context.message.channel_id)
                .content(ids_fmt)?
                .attachment("grid.jpg", encoded)
                .await?,
        );
    }

    if let Some(reply) = reply {
        Ok(Response::Message(reply))
    } else {
        Ok(Response::None)
    }
}

pub async fn pick(context: &mut MessageContext) -> Result<Response> {
    let guild_id = context.message.guild_id.unwrap();
    let now = Utc::now();

    if let Some(message_id) = context.next() {
        let image = sqlx::query_as!(
            Image,
            "SELECT * FROM images WHERE
            (message_id = $1);",
            message_id
        )
        .fetch_optional(&context.pool)
        .await?;

        if let Some(image) = image {
            context
                .http
                .update_guild(guild_id)
                .icon(format!("data:image/png;base64,{}", base64::encode(image.image)))
                .await?;

            // this counts as a rotate, so we tell redis
            let mut redis = context.redis.get().await;
            redis
                .hset("rr-rs:rotations", guild_id.to_string(), now.timestamp().to_string())
                .await?;

            context.react(ResponseReaction::Success.value()).await?;
            Ok(Response::Reaction)
        } else {
            let reply = context.reply("Could not find this image.").await?;
            Ok(Response::Message(reply))
        }
    } else {
        let reply = context.reply("Please specify an image. Try `katze rotate list`.").await?;
        Ok(Response::Message(reply))
    }
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
    let partial_image = partial_images.choose(&mut rand::thread_rng()).unwrap();

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

pub async fn show(context: &mut MessageContext) -> Result<Response> {
    if let Some(message_id) = context.next() {
        let image = sqlx::query_as!(
            Image,
            "SELECT * FROM images WHERE
            (message_id = $1);",
            message_id
        )
        .fetch_optional(&context.pool)
        .await?;

        if let Some(image) = image {
            let reply = context
                .http
                .create_message(context.message.channel_id)
                .content(format!("`{}`", image.message_id))?
                .attachment(format!("{}.{}", image.message_id, image.filetype), image.image)
                .await?;

            Ok(Response::Message(reply))
        } else {
            let reply = context.reply(format!("Image `{}` not found.", message_id)).await?;
            Ok(Response::Message(reply))
        }
    } else {
        let reply = context.reply("No image specified.").await?;
        Ok(Response::Message(reply))
    }
}

pub async fn execute(context: &mut MessageContext) -> Result<Response> {
    if let Some(command) = context.next() {
        match command.as_ref() {
            "add_image" | "pls" => add_image(context).await,
            "count" => count(context).await,
            "delete" | "remove" | "rm" => delete(context).await,
            "list" | "ls" => list(context).await,
            "pick" => pick(context).await,
            "show" => show(context).await,
            _ => Ok(Response::None),
        }
    } else {
        rotate(context).await
    }
}
