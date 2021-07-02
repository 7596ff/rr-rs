pub mod id;
pub mod primitive;

use self::id::{SqlxChannelId, SqlxEmojiId, SqlxGuildId, SqlxMessageId, SqlxRoleId, SqlxUserId};
use chrono::NaiveDateTime;

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
pub struct Movie {
    pub guild_id: SqlxGuildId,
    pub member_id: SqlxUserId,
    pub id: i32,
    pub title: String,
    pub url: Option<String>,
    pub watch_date: Option<NaiveDateTime>,
    pub nominated: bool,
    pub final_votes: i32,
}

#[derive(Debug)]
pub struct MovieVote {
    pub guild_id: SqlxGuildId,
    pub member_id: SqlxUserId,
    pub id: i32,
}

#[derive(Debug)]
pub struct MovieSeq {
    pub id: i32,
    pub seq: i32,
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
