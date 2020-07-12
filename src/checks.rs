use std::fmt::{Display, Formatter, Result as FmtResult};

use anyhow::Result;
use twilight::model::id::RoleId;

use crate::{
    model::{MessageContext, SettingRole},
    table::Setting,
};

#[derive(Debug)]
pub enum CheckError {
    NotOwner,
    MissingRole(SettingRole),
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
    if maybe_role.len() > 1 {
        let role = RoleId::from(maybe_role.parse::<u64>()?);

        let member = context
            .cache
            .member(context.message.guild_id.unwrap(), context.message.author.id)
            .await?;

        // is the role present in the member's roles?
        if member.is_some() && !member.unwrap().roles.contains(&role) {
            return Err(CheckError::MissingRole(setting_role).into());
        }
    }

    Ok(())
}
