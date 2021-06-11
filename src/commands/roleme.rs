use anyhow::Result;
use twilight_http::request::AuditLogReason;

use crate::{
    model::{MessageContext, Response, ResponseReaction},
    table::{raw::RawRolemeRole, RolemeRole},
};

async fn roles(context: &MessageContext) -> Result<Vec<RolemeRole>> {
    let rows = context
        .postgres
        .query(
            "SELECT * FROM roleme_roles WHERE
            (guild_id = $1)",
            &[&context.message.guild_id.unwrap().to_string()],
        )
        .await?;

    // cache doesn't handle the roles correctly. we need to get them on each call
    let http_roles = context
        .http
        .roles(context.message.guild_id.unwrap())
        .await?;

    let raw: Vec<RawRolemeRole> = serde_postgres::from_rows(&rows)?;

    let (roles, stale_roles): (Vec<RolemeRole>, Vec<RolemeRole>) = raw
        .into_iter()
        .map(RolemeRole::from)
        .partition(|r| http_roles.iter().any(|hr| hr.id == r.id));

    // purge stale roles if they are not in the cache
    for role in stale_roles {
        context
            .postgres
            .execute(
                "DELETE FROM roleme_roles WHERE
                (id = $1);",
                &[&role.id.to_string()],
            )
            .await?;
    }

    Ok(roles)
}

async fn add(context: &mut MessageContext) -> Result<Response> {
    let roles = roles(&context).await?;

    let maybe_id = context.message.mention_roles.first();
    let name = context.args.join(" ");

    let candidate = roles
        .iter()
        .filter_map(|r| context.cache.role(r.id))
        .find(|role| {
            if let Some(id) = maybe_id {
                role.id == *id
            } else {
                role.name == name
            }
        });

    if let Some(role) = candidate {
        context
            .http
            .add_guild_member_role(
                context.message.guild_id.unwrap(),
                context.message.author.id,
                role.id,
            )
            .reason("Automated roleme role grant")?
            .await?;

        context.react(ResponseReaction::Success.value()).await?;

        Ok(Response::Reaction)
    } else {
        let reply = context
            .reply("Couldn't find that role in the list of roleme roles.")
            .await?;

        Ok(Response::Message(reply))
    }
}

async fn create(context: &mut MessageContext) -> Result<Response> {
    let role = context
        .http
        .create_role(context.message.guild_id.unwrap())
        .name(context.args.join(" "))
        .await?;

    context
        .postgres
        .execute(
            "INSERT INTO roleme_roles (guild_id, id) VALUES
            ($1, $2);",
            &[
                &context.message.guild_id.unwrap().to_string(),
                &role.id.to_string(),
            ],
        )
        .await?;

    context.react(ResponseReaction::Success.value()).await?;

    Ok(Response::Reaction)
}

async fn disable(context: &mut MessageContext) -> Result<Response> {
    let roles = roles(&context).await?;

    let maybe_id = context.message.mention_roles.first();
    let name = context.args.join(" ");

    let candidate = roles
        .iter()
        .filter_map(|r| context.cache.role(r.id))
        .find(|role| {
            if let Some(id) = maybe_id {
                role.id == *id
            } else {
                role.name == name
            }
        });

    if let Some(role) = candidate {
        context
            .postgres
            .execute(
                "DELETE FROM roleme_roles WHERE id = $1;",
                &[&role.id.to_string()],
            )
            .await?;

        context.react(ResponseReaction::Success.value()).await?;

        Ok(Response::Reaction)
    } else {
        let reply = context
            .reply("This role wasn't found in the list of roleme roles.")
            .await?;

        Ok(Response::Message(reply))
    }
}

async fn enable(context: &mut MessageContext) -> Result<Response> {
    let maybe_id = context.message.mention_roles.first();
    let name = context.args.join(" ");

    let roles = context
        .http
        .roles(context.message.guild_id.unwrap())
        .await?;

    let candidate = roles.iter().find(|r| {
        if let Some(id) = maybe_id {
            r.id == *id
        } else {
            r.name == name
        }
    });

    if let Some(role) = candidate {
        context
            .postgres
            .execute(
                "INSERT INTO roleme_roles (guild_id, id)
                VALUES ($1, $2);",
                &[
                    &context.message.guild_id.unwrap().to_string(),
                    &role.id.to_string(),
                ],
            )
            .await?;

        context.react(ResponseReaction::Success.value()).await?;

        Ok(Response::Reaction)
    } else {
        let reply = context
            .reply("This role wasn't found in the server.")
            .await?;

        Ok(Response::Message(reply))
    }
}

async fn remove(context: &mut MessageContext) -> Result<Response> {
    let roles = roles(&context).await?;

    let maybe_id = context.message.mention_roles.first();
    let name = context.args.join(" ");

    let candidate = roles
        .iter()
        .filter_map(|r| context.cache.role(r.id))
        .find(|role| {
            if let Some(id) = maybe_id {
                role.id == *id
            } else {
                role.name == name
            }
        });

    if let Some(role) = candidate {
        context
            .http
            .remove_guild_member_role(
                context.message.guild_id.unwrap(),
                context.message.author.id,
                role.id,
            )
            .reason("Automated roleme role removal")?
            .await?;

        context.react(ResponseReaction::Success.value()).await?;

        Ok(Response::Reaction)
    } else {
        let reply = context
            .reply("Couldn't find that role in the list of roleme roles.")
            .await?;

        Ok(Response::Message(reply))
    }
}

async fn list(context: &mut MessageContext) -> Result<Response> {
    let roles = roles(&context).await?;

    if roles.is_empty() {
        let reply = context
            .reply("There aren't any roleme roles in this server.")
            .await?;

        Ok(Response::Message(reply))
    } else {
        let roles_fmt = roles
            .into_iter()
            .filter_map(|r| context.cache.role(r.id))
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

pub async fn execute(context: &mut MessageContext) -> Result<Response> {
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
