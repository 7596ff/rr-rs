use sqlx::{
    database::HasValueRef, decode::Decode, error::BoxDynError, postgres::PgTypeInfo, Postgres, Type,
};
use std::fmt::{Display, Formatter, Result as FmtResult};
use twilight_model::id::{ChannelId, EmojiId, GuildId, MessageId, RoleId, UserId};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SqlxChannelId(pub ChannelId);

impl Display for SqlxChannelId {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.0.fmt(f)
    }
}

impl<'r> Decode<'r, Postgres> for SqlxChannelId {
    fn decode(value: <Postgres as HasValueRef<'r>>::ValueRef) -> Result<Self, BoxDynError> {
        let value = <&str as Decode<Postgres>>::decode(value)?;

        let value = value.parse::<u64>().map_err(Box::new)?;

        Ok(Self(ChannelId(value)))
    }
}

impl PartialEq<ChannelId> for SqlxChannelId {
    fn eq(&self, other: &ChannelId) -> bool {
        &self.0 == other
    }
}

impl Type<Postgres> for SqlxChannelId {
    fn type_info() -> PgTypeInfo {
        <&str as Type<Postgres>>::type_info()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SqlxEmojiId(pub EmojiId);

impl Display for SqlxEmojiId {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.0.fmt(f)
    }
}

impl<'r> Decode<'r, Postgres> for SqlxEmojiId {
    fn decode(value: <Postgres as HasValueRef<'r>>::ValueRef) -> Result<Self, BoxDynError> {
        let value = <&str as Decode<Postgres>>::decode(value)?;

        let value = value.parse::<u64>().map_err(Box::new)?;

        Ok(Self(EmojiId(value)))
    }
}

impl PartialEq<EmojiId> for SqlxEmojiId {
    fn eq(&self, other: &EmojiId) -> bool {
        &self.0 == other
    }
}

impl Type<Postgres> for SqlxEmojiId {
    fn type_info() -> PgTypeInfo {
        <&str as Type<Postgres>>::type_info()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SqlxGuildId(pub GuildId);

impl Display for SqlxGuildId {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.0.fmt(f)
    }
}

impl<'r> Decode<'r, Postgres> for SqlxGuildId {
    fn decode(value: <Postgres as HasValueRef<'r>>::ValueRef) -> Result<Self, BoxDynError> {
        let value = <&str as Decode<Postgres>>::decode(value)?;

        let value = value.parse::<u64>().map_err(Box::new)?;

        Ok(Self(GuildId(value)))
    }
}

impl PartialEq<GuildId> for SqlxGuildId {
    fn eq(&self, other: &GuildId) -> bool {
        &self.0 == other
    }
}

impl Type<Postgres> for SqlxGuildId {
    fn type_info() -> PgTypeInfo {
        <&str as Type<Postgres>>::type_info()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SqlxMessageId(pub MessageId);

impl Display for SqlxMessageId {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.0.fmt(f)
    }
}

impl<'r> Decode<'r, Postgres> for SqlxMessageId {
    fn decode(value: <Postgres as HasValueRef<'r>>::ValueRef) -> Result<Self, BoxDynError> {
        let value = <&str as Decode<Postgres>>::decode(value)?;

        let value = value.parse::<u64>().map_err(Box::new)?;

        Ok(Self(MessageId(value)))
    }
}

impl PartialEq<MessageId> for SqlxMessageId {
    fn eq(&self, other: &MessageId) -> bool {
        &self.0 == other
    }
}

impl Type<Postgres> for SqlxMessageId {
    fn type_info() -> PgTypeInfo {
        <&str as Type<Postgres>>::type_info()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SqlxRoleId(pub RoleId);

impl Display for SqlxRoleId {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.0.fmt(f)
    }
}

impl<'r> Decode<'r, Postgres> for SqlxRoleId {
    fn decode(value: <Postgres as HasValueRef<'r>>::ValueRef) -> Result<Self, BoxDynError> {
        let value = <&str as Decode<Postgres>>::decode(value)?;

        let value = value.parse::<u64>().map_err(Box::new)?;

        Ok(Self(RoleId(value)))
    }
}

impl PartialEq<RoleId> for SqlxRoleId {
    fn eq(&self, other: &RoleId) -> bool {
        &self.0 == other
    }
}

impl Type<Postgres> for SqlxRoleId {
    fn type_info() -> PgTypeInfo {
        <&str as Type<Postgres>>::type_info()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SqlxUserId(pub UserId);

impl Display for SqlxUserId {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.0.fmt(f)
    }
}

impl<'r> Decode<'r, Postgres> for SqlxUserId {
    fn decode(value: <Postgres as HasValueRef<'r>>::ValueRef) -> Result<Self, BoxDynError> {
        let value = <&str as Decode<Postgres>>::decode(value)?;

        let value = value.parse::<u64>().map_err(Box::new)?;

        Ok(Self(UserId(value)))
    }
}

impl PartialEq<UserId> for SqlxUserId {
    fn eq(&self, other: &UserId) -> bool {
        &self.0 == other
    }
}

impl Type<Postgres> for SqlxUserId {
    fn type_info() -> PgTypeInfo {
        <&str as Type<Postgres>>::type_info()
    }
}
