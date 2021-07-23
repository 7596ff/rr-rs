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
