pub mod id;
pub mod primitive;

use self::id::{SqlxChannelId, SqlxEmojiId, SqlxGuildId, SqlxMessageId, SqlxRoleId, SqlxUserId};
use chrono::NaiveDateTime;
use sqlx::{Error as SqlxError, PgPool};
use std::{future::Future, pin::Pin};
use twilight_model::id::GuildId;

#[derive(Debug)]
pub struct Guild {
    pub id: SqlxGuildId,
    pub name: String,
}

#[derive(Debug)]
pub struct Setting {
    pub guild_id: SqlxGuildId,
    pub starboard_channel_id: Option<SqlxChannelId>,
    pub starboard_emoji: String,
    pub starboard_min_stars: i32,
    pub movies_role: Option<SqlxRoleId>,
    pub rotate_every: i32,
    pub rotate_enabled: bool,
    pub vtrack: bool,
}

impl Setting {
    pub fn query(
        pool: PgPool,
        guild_id: GuildId,
    ) -> Pin<Box<dyn Future<Output = Result<Self, SqlxError>> + Send>> {
        Box::pin(async move {
            sqlx::query_as!(
                Self,
                "SELECT
                    guild_id AS \"guild_id: _\",
                    starboard_channel_id AS \"starboard_channel_id: _\",
                    starboard_emoji,
                    starboard_min_stars,
                    movies_role AS \"movies_role: _\",
                    rotate_every,
                    rotate_enabled,
                    vtrack
                FROM settings WHERE (guild_id = $1);",
                guild_id.to_string(),
            )
            .fetch_one(&pool)
            .await
        })
    }
}

#[derive(Debug)]
pub struct InviteRole {
    pub guild_id: SqlxGuildId,
    pub id: SqlxRoleId,
    pub invite_code: String,
}

#[derive(Debug)]
pub struct RolemeRole {
    pub guild_id: SqlxGuildId,
    pub id: SqlxRoleId,
    pub color: Option<String>,
}

#[derive(Debug)]
pub struct StarboardEntry {
    pub guild_id: SqlxGuildId,
    pub member_id: SqlxUserId,
    pub channel_id: SqlxChannelId,
    pub message_id: SqlxMessageId,
    pub post_id: SqlxMessageId,
    pub star_count: i32,
    pub date: NaiveDateTime,
}

#[derive(Debug)]
pub struct Image {
    pub guild_id: SqlxGuildId,
    pub message_id: SqlxMessageId,
    pub image: Vec<u8>,
    pub filetype: String,
}

#[derive(Debug)]
pub struct Emoji {
    pub datetime: i64,
    pub guild_id: SqlxGuildId,
    pub message_id: SqlxMessageId,
    pub member_id: SqlxUserId,
    pub emoji_id: SqlxEmojiId,
    pub reaction: bool,
}
