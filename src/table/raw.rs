use chrono::NaiveDateTime;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RawGuild {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct RawSetting {
    pub guild_id: String,
    pub starboard_channel_id: Option<String>,
    pub starboard_emoji: String,
    pub starboard_min_stars: i32,
    pub movies_role: Option<String>,
    pub rotate_every: i32,
    pub rotate_enabled: bool,
    pub vtrack: bool,
}

#[derive(Debug, Deserialize)]
pub struct RawInviteRole {
    pub guild_id: String,
    pub id: String,
    pub invite_code: String,
}

#[derive(Debug, Deserialize)]
pub struct RawRolemeRole {
    pub guild_id: String,
    pub id: String,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RawStarboardEntry {
    pub guild_id: String,
    pub member_id: String,
    pub channel_id: String,
    pub message_id: String,
    pub post_id: String,
    pub star_count: i32,
    pub date: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct RawMovie {
    pub guild_id: String,
    pub member_id: String,
    pub id: i32,
    pub title: String,
    pub url: Option<String>,
    pub watch_date: Option<NaiveDateTime>,
    pub nominated: bool,
    pub final_votes: i32,
}

#[derive(Debug, Deserialize)]
pub struct RawMovieVote {
    pub guild_id: String,
    pub member_id: String,
    pub id: i32,
}

#[derive(Debug, Deserialize)]
pub struct RawImage {
    pub guild_id: String,
    pub message_id: String,
    pub image: Vec<u8>,
    pub filetype: String,
}

#[derive(Debug, Deserialize)]
pub struct RawEmoji {
    pub datetime: i64,
    pub guild_id: String,
    pub message_id: String,
    pub member_id: String,
    pub emoji_id: String,
    pub reaction: bool,
}
