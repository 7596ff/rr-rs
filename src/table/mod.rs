pub mod raw;

use chrono::NaiveDateTime;
use twilight_model::id::{ChannelId, EmojiId, GuildId, MessageId, RoleId, UserId};

#[derive(Debug)]
pub struct Guild {
    pub id: GuildId,
    pub name: String,
}

impl From<raw::RawGuild> for Guild {
    fn from(other: raw::RawGuild) -> Self {
        Self { id: GuildId(other.id.parse::<u64>().unwrap()), name: other.name }
    }
}

#[derive(Debug)]
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

impl From<raw::RawSetting> for Setting {
    fn from(other: raw::RawSetting) -> Self {
        Self {
            guild_id: GuildId(other.guild_id.parse::<u64>().unwrap()),
            starboard_channel_id: other
                .starboard_channel_id
                .map(|id| ChannelId(id.parse::<u64>().unwrap())),
            starboard_emoji: other.starboard_emoji,
            starboard_min_stars: other.starboard_min_stars,
            movies_role: other.movies_role.map(|r| RoleId(r.parse::<u64>().unwrap())),
            rotate_every: other.rotate_every,
            rotate_enabled: other.rotate_enabled,
            vtrack: other.vtrack,
        }
    }
}

#[derive(Debug)]
pub struct InviteRole {
    pub guild_id: GuildId,
    pub id: RoleId,
    pub invite_code: String,
}

impl From<raw::RawInviteRole> for InviteRole {
    fn from(other: raw::RawInviteRole) -> Self {
        Self {
            guild_id: GuildId(other.guild_id.parse::<u64>().unwrap()),
            id: RoleId(other.id.parse::<u64>().unwrap()),
            invite_code: other.invite_code,
        }
    }
}

#[derive(Debug)]
pub struct RolemeRole {
    pub guild_id: GuildId,
    pub id: RoleId,
    pub color: Option<String>,
}

impl From<raw::RawRolemeRole> for RolemeRole {
    fn from(other: raw::RawRolemeRole) -> Self {
        Self {
            guild_id: GuildId(other.guild_id.parse::<u64>().unwrap()),
            id: RoleId(other.id.parse::<u64>().unwrap()),
            color: other.color,
        }
    }
}

#[derive(Debug)]
pub struct StarboardEntry {
    pub guild_id: GuildId,
    pub member_id: UserId,
    pub channel_id: ChannelId,
    pub message_id: MessageId,
    pub post_id: MessageId,
    pub star_count: i32,
    pub date: NaiveDateTime,
}

impl From<raw::RawStarboardEntry> for StarboardEntry {
    fn from(other: raw::RawStarboardEntry) -> Self {
        Self {
            guild_id: GuildId(other.guild_id.parse::<u64>().unwrap()),
            member_id: UserId(other.member_id.parse::<u64>().unwrap()),
            channel_id: ChannelId(other.channel_id.parse::<u64>().unwrap()),
            message_id: MessageId(other.message_id.parse::<u64>().unwrap()),
            post_id: MessageId(other.post_id.parse::<u64>().unwrap()),
            star_count: other.star_count,
            date: other.date,
        }
    }
}

#[derive(Debug)]
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

impl From<raw::RawMovie> for Movie {
    fn from(other: raw::RawMovie) -> Self {
        Self {
            guild_id: GuildId(other.guild_id.parse::<u64>().unwrap()),
            member_id: UserId(other.member_id.parse::<u64>().unwrap()),
            id: other.id,
            title: other.title,
            url: other.url,
            watch_date: other.watch_date,
            nominated: other.nominated,
            final_votes: other.final_votes,
        }
    }
}

#[derive(Debug)]
pub struct MovieVote {
    pub guild_id: GuildId,
    pub member_id: UserId,
    pub id: i32,
}

impl From<raw::RawMovieVote> for MovieVote {
    fn from(other: raw::RawMovieVote) -> Self {
        Self {
            guild_id: GuildId(other.guild_id.parse::<u64>().unwrap()),
            member_id: UserId(other.member_id.parse::<u64>().unwrap()),
            id: other.id,
        }
    }
}

#[derive(Debug)]
pub struct MovieSeq {
    pub id: i32,
    pub seq: i32,
}

#[derive(Debug)]
pub struct Image {
    pub guild_id: GuildId,
    pub message_id: MessageId,
    pub image: Vec<u8>,
    pub filetype: String,
}

impl From<raw::RawImage> for Image {
    fn from(other: raw::RawImage) -> Self {
        Self {
            guild_id: GuildId(other.guild_id.parse::<u64>().unwrap()),
            message_id: MessageId(other.message_id.parse::<u64>().unwrap()),
            image: other.image,
            filetype: other.filetype,
        }
    }
}

#[derive(Debug)]
pub struct Emoji {
    pub datetime: i64,
    pub guild_id: GuildId,
    pub message_id: MessageId,
    pub member_id: UserId,
    pub emoji_id: EmojiId,
    pub reaction: bool,
}

impl From<raw::RawEmoji> for Emoji {
    fn from(other: raw::RawEmoji) -> Self {
        Self {
            datetime: other.datetime,
            guild_id: GuildId(other.guild_id.parse::<u64>().unwrap()),
            message_id: MessageId(other.message_id.parse::<u64>().unwrap()),
            member_id: UserId(other.member_id.parse::<u64>().unwrap()),
            emoji_id: EmojiId(other.emoji_id.parse::<u64>().unwrap()),
            reaction: other.reaction,
        }
    }
}
