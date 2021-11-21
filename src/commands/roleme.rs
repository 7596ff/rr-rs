use crate::{
    model::{GenericError, MessageContext, Response, ResponseReaction},
    table::RolemeRole,
};
use anyhow::anyhow;
use twilight_http::request::AuditLogReason;

async fn roles(context: &MessageContext) -> Result<Vec<RolemeRole>, GenericError> {
    let roles = sqlx::query_as!(
        RolemeRole,
        "SELECT
            guild_id AS \"guild_id: _\",
            id AS \"id: _\",
            color
        FROM roleme_roles WHERE
        (guild_id = $1);",
        context.message.guild_id.unwrap().to_string(),
    )
    .fetch_all(context.postgres())
    .await?;

    let cached = context
        .cache()
        .guild_roles(context.message.guild_id.unwrap())
        .ok_or_else(|| anyhow!("guild not found"))?;

    let (roles, stale_roles): (Vec<RolemeRole>, Vec<RolemeRole>) = roles
        .into_iter()
        .partition(|r| cached.iter().any(|c| r.id.eq(c)));

    // purge stale roles if they are not in the cache
    for role in stale_roles {
        sqlx::query!(
            "DELETE FROM roleme_roles WHERE
            (id = $1);",
            role.id.to_string(),
        )
        .execute(context.postgres())
        .await?;
    }

    Ok(roles)
}

async fn add(context: &mut MessageContext) -> Result<Response, GenericError> {
    let roles = roles(context).await?;

    let maybe_id = context.message.mention_roles.first();
    let name = context.args.join(" ");

    let candidate = roles
        .iter()
        .filter_map(|r| context.cache().role(r.id.0))
        .find(|role| {
            if let Some(id) = maybe_id {
                role.id == *id
            } else {
                role.name == name
            }
        });

    if let Some(role) = candidate {
        context
            .http()
            .add_guild_member_role(
                context.message.guild_id.unwrap(),
                context.message.author.id,
                role.id,
            )
            .reason("Automated roleme role grant")?
            .exec()
            .await?;

        context.react(&ResponseReaction::Success.value()).await?;

        Ok(Response::Reaction)
    } else {
        let reply = context
            .reply("Couldn't find that role in the list of roleme roles.")
            .await?;

        Ok(Response::Message(reply))
    }
}

async fn create(context: &mut MessageContext) -> Result<Response, GenericError> {
    let name = context.args.join(" ");

    let role = context
        .http()
        .create_role(context.message.guild_id.unwrap())
        .name(&name)
        .exec()
        .await?
        .model()
        .await?;

    sqlx::query!(
        "INSERT INTO roleme_roles (guild_id, id) VALUES
        ($1, $2);",
        context.message.guild_id.unwrap().to_string(),
        role.id.to_string(),
    )
    .execute(context.postgres())
    .await?;

    context.react(&ResponseReaction::Success.value()).await?;

    Ok(Response::Reaction)
}

async fn disable(context: &mut MessageContext) -> Result<Response, GenericError> {
    let roles = roles(context).await?;

    let maybe_id = context.message.mention_roles.first();
    let name = context.args.join(" ");

    let candidate = roles
        .iter()
        .filter_map(|r| context.cache().role(r.id.0))
        .find(|role| {
            if let Some(id) = maybe_id {
                role.id == *id
            } else {
                role.name == name
            }
        });

    if let Some(role) = candidate {
        sqlx::query!(
            "DELETE FROM roleme_roles WHERE id = $1;",
            role.id.to_string(),
        )
        .execute(context.postgres())
        .await?;

        context.react(&ResponseReaction::Success.value()).await?;

        Ok(Response::Reaction)
    } else {
        let reply = context
            .reply("This role wasn't found in the list of roleme roles.")
            .await?;

        Ok(Response::Message(reply))
    }
}

async fn enable(context: &mut MessageContext) -> Result<Response, GenericError> {
    let maybe_id = context.message.mention_roles.first();
    let name = context.args.join(" ");

    let roles = context
        .http()
        .roles(context.message.guild_id.unwrap())
        .exec()
        .await?
        .models()
        .await?;

    let candidate = roles.iter().find(|r| {
        if let Some(id) = maybe_id {
            r.id == *id
        } else {
            r.name == name
        }
    });

    if let Some(role) = candidate {
        sqlx::query!(
            "INSERT INTO roleme_roles (guild_id, id)
            VALUES ($1, $2);",
            context.message.guild_id.unwrap().to_string(),
            role.id.to_string(),
        )
        .execute(context.postgres())
        .await?;

        context.react(&ResponseReaction::Success.value()).await?;

        Ok(Response::Reaction)
    } else {
        let reply = context
            .reply("This role wasn't found in the server.")
            .await?;

        Ok(Response::Message(reply))
    }
}

async fn remove(context: &mut MessageContext) -> Result<Response, GenericError> {
    let roles = roles(context).await?;

    let maybe_id = context.message.mention_roles.first();
    let name = context.args.join(" ");

    let candidate = roles
        .iter()
        .filter_map(|r| context.cache().role(r.id.0))
        .find(|role| {
            if let Some(id) = maybe_id {
                role.id == *id
            } else {
                role.name == name
            }
        });

    if let Some(role) = candidate {
        context
            .http()
            .remove_guild_member_role(
                context.message.guild_id.unwrap(),
                context.message.author.id,
                role.id,
            )
            .reason("Automated roleme role removal")?
            .exec()
            .await?;

        context.react(&ResponseReaction::Success.value()).await?;

        Ok(Response::Reaction)
    } else {
        let reply = context
            .reply("Couldn't find that role in the list of roleme roles.")
            .await?;

        Ok(Response::Message(reply))
    }
}

async fn list(context: &mut MessageContext) -> Result<Response, GenericError> {
    let roles = roles(context).await?;

    if roles.is_empty() {
        let reply = context
            .reply("There aren't any roleme roles in this server.")
            .await?;

        Ok(Response::Message(reply))
    } else {
        let roles_fmt = roles
            .into_iter()
            .filter_map(|r| context.cache().role(r.id.0))
            .map(|r| format!("* `{}`", r.name))
            .collect::<Vec<String>>()
            .join("\n");

        let reply = context
            .reply(format!(
                "Here's a list of roles you can give yourself in this server:\n{}",
                roles_fmt
            ))
            .await?;

        Ok(Response::Message(reply))
    }
}

pub async fn execute(context: &mut MessageContext) -> Result<Response, GenericError> {
    if let Some(command) = context.next() {
        match command.as_ref() {
            "add" => add(context).await,
            "create" => create(context).await,
            "disable" => disable(context).await,
            "enable" => enable(context).await,
            "remove" => remove(context).await,
            "list" => list(context).await,
            _ => add(context).await,
        }
    } else {
        add(context).await
    }
}
