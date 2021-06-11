use chrono::NaiveDateTime;
use serde::Deserialize;
use twilight_model::id::{ChannelId, EmojiId, GuildId, MessageId, RoleId, UserId};

#[derive(Debug, Deserialize)]
pub struct Guild {
    pub id: GuildId,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct Setting {
    pub guild_id: GuildId,
    pub starboard_channel_id: Option<ChannelId>,
    pub starboard_emoji: String,
    pub starboard_min_stars: i32,
    pub movies_role: Option<RoleId>,
    pub rotate_every: i32,
    pub rotate_enabled: bool,
    pub vtrack: bool,
}

#[derive(Debug, Deserialize)]
pub struct InviteRole {
    pub guild_id: GuildId,
    pub id: RoleId,
    pub invite_code: String,
}

#[derive(Debug, Deserialize)]
pub struct RolemeRole {
    pub guild_id: GuildId,
    pub id: RoleId,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StarboardEntry {
    pub guild_id: GuildId,
    pub member_id: UserId,
    pub channel_id: ChannelId,
    pub message_id: MessageId,
    pub post_id: MessageId,
    pub star_count: i32,
    pub date: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct Movie {
    pub guild_id: GuildId,
    pub member_id: UserId,
    pub id: i32,
    pub title: String,
    pub url: Option<String>,
    pub watch_date: Option<NaiveDateTime>,
    pub nominated: bool,
    pub final_votes: i32,
}

#[derive(Debug, Deserialize)]
pub struct MovieVote {
    pub guild_id: GuildId,
    pub member_id: UserId,
    pub id: i32,
}

#[derive(Debug)]
pub struct MovieSeq {
    pub id: i32,
    pub seq: i32,
}

#[derive(Debug, Deserialize)]
pub struct Image {
    pub guild_id: GuildId,
    pub message_id: MessageId,
    pub image: Vec<u8>,
    pub filetype: String,
}

#[derive(Debug, Deserialize)]
pub struct Emoji {
    pub datetime: i64,
    pub guild_id: GuildId,
    pub message_id: MessageId,
    pub member_id: UserId,
    pub emoji_id: EmojiId,
    pub reaction: bool,
}
