use chrono::NaiveDateTime;

#[derive(Debug)]
pub struct Guild {
    pub id: String,
    pub name: String,
}

#[derive(Debug)]
pub struct Setting {
    pub guild_id: String,
    pub starboard_channel_id: Option<String>,
    pub starboard_emoji: String,
    pub starboard_min_stars: i32,
    pub movies_role: Option<String>,
    pub rotate_every: i32,
}

#[derive(Debug)]
pub struct InviteRole {
    pub guild_id: String,
    pub id: String,
    pub invite_code: String,
}

#[derive(Debug)]
pub struct RolemeRole {
    pub guild_id: String,
    pub id: String,
    pub color: Option<String>,
}

#[derive(Debug)]
pub struct StarboardEntry {
    pub guild_id: String,
    pub member_id: String,
    pub channel_id: String,
    pub message_id: String,
    pub post_id: String,
    pub star_count: i32,
    pub date: NaiveDateTime,
}

#[derive(Debug)]
pub struct Movie {
    pub guild_id: String,
    pub member_id: String,
    pub id: i32,
    pub title: String,
    pub url: Option<String>,
    pub watch_date: Option<NaiveDateTime>,
    pub nominated: bool,
    pub final_votes: i32,
}

#[derive(Debug)]
pub struct MovieVote {
    pub guild_id: String,
    pub member_id: String,
    pub id: i32,
}

#[derive(Debug)]
pub struct MovieSeq {
    pub id: i32,
    pub seq: i32,
}

#[derive(Debug)]
pub struct Image {
    pub guild_id: String,
    pub message_id: String,
    pub image: Vec<u8>,
    pub filetype: String,
}
