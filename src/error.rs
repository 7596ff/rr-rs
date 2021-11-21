use crate::model::GenericError;
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};
use twilight_model::id::GuildId;

#[derive(Debug)]
pub enum KatzeError {
    GuildNotFound { id: GuildId },
    NoEmojiFound,
    NoMatchingEmojis,
}

impl KatzeError {
    pub fn guild_not_found(id: GuildId) -> GenericError {
        Box::new(KatzeError::GuildNotFound { id })
    }

    pub fn no_emoji_found() -> GenericError {
        Box::new(KatzeError::NoEmojiFound)
    }

    pub fn no_matching_emojis() -> GenericError {
        Box::new(KatzeError::NoMatchingEmojis)
    }
}

impl Display for KatzeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::GuildNotFound { id } => {
                f.write_str("guild not found: ")?;

                Display::fmt(id, f)
            }
            Self::NoEmojiFound => f.write_str("no emoji id found"),
            Self::NoMatchingEmojis => f.write_str("no matching emojis in supplied text"),
        }
    }
}

impl Error for KatzeError {}
