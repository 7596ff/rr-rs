use crate::model::MessageContext;
use anyhow::{Error as Anyhow, Result};
use std::fmt::{Display, Formatter, Result as FmtResult};
use twilight_model::{guild::Permissions, id::RoleId};
use twilight_util::permission_calculator::PermissionCalculator;

#[derive(Debug)]
pub enum CheckError {
    MissingPermissions(Permissions),
    NoGuild,
    NotOwner,
}

impl std::error::Error for CheckError {}

impl Display for CheckError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Self::MissingPermissions(permissions) => {
                write!(f, "You are missing permissions: {:?}.", permissions)
            }
            Self::NoGuild => f.write_str("No guild id"),
            Self::NotOwner => write!(f, "You are not the owner."),
        }
    }
}

pub fn is_owner(context: &MessageContext) -> Result<()> {
    if dotenv::var("OWNER")?.parse::<u64>()? == context.message.author.id.0 {
        Ok(())
    } else {
        Err(CheckError::NotOwner.into())
    }
}

// there is no in memory cache guild roles, so we just pretend the only guild roles are the ones
// the member has.
pub async fn has_permission(context: &MessageContext, permissions: Permissions) -> Result<()> {
    let guild_id = match context.message.guild_id {
        Some(guild_id) => guild_id,
        None => return Err(Anyhow::new(CheckError::NoGuild)),
    };

    if let (Some(member), Some(guild), Some(everyone_role)) = (
        context.cache.member(guild_id, context.message.author.id),
        context.cache.guild(guild_id),
        context.cache.role(RoleId(guild_id.0)),
    ) {
        let roles = member
            .roles
            .iter()
            .filter_map(|id| context.cache.role(*id))
            .map(|role| (role.id, role.permissions))
            .collect::<Vec<_>>();

        let calculator = PermissionCalculator::new(
            guild_id,
            context.message.author.id,
            everyone_role.permissions,
            roles.as_slice(),
        )
        .owner_id(guild.owner_id)
        .root();

        if calculator.contains(permissions) {
            return Ok(());
        }
    }

    Err(CheckError::MissingPermissions(permissions).into())
}
