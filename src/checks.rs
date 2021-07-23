use crate::{
    model::{MessageContext, SettingRole},
    table::Setting,
};
use anyhow::{Error as Anyhow, Result};
use std::fmt::{Display, Formatter, Result as FmtResult};
use twilight_model::{guild::Permissions, id::RoleId};
use twilight_util::permission_calculator::PermissionCalculator;

#[derive(Debug)]
pub enum CheckError {
    MissingPermissions(Permissions),
    MissingRole(SettingRole),
    NoGuild,
    NotOwner,
}

impl std::error::Error for CheckError {}

impl Display for CheckError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Self::MissingRole(setting_role) => write!(
                f,
                "You are missing the role: `{}`. Please ask a server administrator for the role.",
                setting_role
            ),
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

// check if the server has a specified role, and if the member has that role.
// if neither, just accept the command.
pub async fn has_role(context: &MessageContext, setting_role: SettingRole) -> Result<()> {
    let setting =
        Setting::query(context.postgres.clone(), context.message.guild_id.unwrap()).await?;

    let maybe_role = match setting_role {
        SettingRole::Movies => setting.movies_role,
    };

    // does the server have a role set?
    if let Some(role) = maybe_role {
        let member = context
            .cache
            .member(context.message.guild_id.unwrap(), context.message.author.id);

        // is the role present in the member's roles?
        if member.is_some() && !member.unwrap().roles.contains(&role.0) {
            return Err(CheckError::MissingRole(setting_role).into());
        }
    }

    Ok(())
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
