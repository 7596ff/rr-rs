use std::collections::HashMap;
use std::fmt::{Display, Formatter, Result as FmtResult};

use anyhow::Result;
use rarity_permission_calculator::Calculator;
use twilight_model::{guild::Permissions, id::RoleId};

use crate::{
    model::{MessageContext, SettingRole},
    table::Setting,
};

#[derive(Debug)]
pub enum CheckError {
    NotOwner,
    MissingRole(SettingRole),
    MissingPermissions(Permissions),
}

impl std::error::Error for CheckError {}

impl Display for CheckError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Self::NotOwner => write!(f, "You are not the owner."),
            Self::MissingRole(setting_role) => write!(
                f,
                "You are missing the role: `{}`. Please ask a server administrator for the role.",
                setting_role
            ),
            Self::MissingPermissions(permissions) => {
                write!(f, "You are missing permissions: {:?}.", permissions)
            }
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
    let settings = sqlx::query_as!(
        Setting,
        "SELECT * FROM settings WHERE
        (guild_id = $1);",
        context.message.guild_id.unwrap().to_string(),
    )
    .fetch_one(&context.pool)
    .await?;

    let maybe_role = match setting_role {
        SettingRole::Movies => settings.movies_role,
    };

    // does the server have a role set?
    if let Some(role) = maybe_role {
        let role = RoleId::from(role.parse::<u64>()?);

        let member =
            context.cache.member(context.message.guild_id.unwrap(), context.message.author.id);

        // is the role present in the member's roles?
        if member.is_some() && !member.unwrap().roles.contains(&role) {
            return Err(CheckError::MissingRole(setting_role).into());
        }
    }

    Ok(())
}

// there is no in memory cache guild roles, so we just pretend the only guild roles are the ones
// the member has.
pub async fn has_permission(context: &MessageContext, permissions: Permissions) -> Result<()> {
    if let Some(member) = &context.message.member {
        let mut roles: HashMap<RoleId, Permissions> = HashMap::new();
        for role_id in member.roles.iter() {
            let cached_role = context.cache.role(role_id.to_owned());
            if let Some(role) = cached_role {
                roles.insert(*role_id, role.permissions);
            }
        }

        // we should know there's a guild at this point
        let cached_guild = context.cache.guild(context.message.guild_id.unwrap()).unwrap();

        let member_permissions = Calculator::new(cached_guild.id, cached_guild.owner_id, &roles)
            .continue_on_missing_items(true)
            .member(context.message.author.id, roles.clone().keys())
            .permissions()?;

        if member_permissions.contains(permissions) {
            return Ok(());
        }
    }

    Err(CheckError::MissingPermissions(permissions).into())
}
