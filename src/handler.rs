use log::{error, info};
use tokio::stream::StreamExt;
use twilight::{
    gateway::Event,
    http::{api_error::ApiError, error::Error as HttpError},
};

use crate::{
    commands,
    model::{EventContext, MessageContext, ReactionContext, Response},
    reactions,
};

fn log_response(context: &MessageContext, response: &Response, command: &str) {
    match response {
        Response::Message(reply) => {
            info!(
                "channel:{} timestamp:{} command:{}",
                reply.channel_id, reply.timestamp, command
            );
        }
        Response::Reaction => {
            info!(
                "channel:{} timestamp:{} command:{}",
                context.message.channel_id, context.message.timestamp, command
            );
        }
        Response::None => {}
    }
}

fn log_error(context: &MessageContext, why: anyhow::Error, command: &str) {
    if let Some(HttpError::Response { error, .. }) = why.downcast_ref::<HttpError>() {
        if let ApiError::General(general) = error {
            error!(
                "channel:{} timestamp:{} command:{}\n{} {}",
                context.message.channel_id,
                context.message.timestamp,
                command,
                general.code,
                general.message,
            );
        }
    } else {
        error!(
            "channel:{} timestamp:{}\nerror processing command:{}\n{:?}",
            context.message.channel_id, context.message.timestamp, command, why
        );
    }
}

pub async fn handle_event(event_context: EventContext) -> anyhow::Result<()> {
    match event_context.event {
        Event::Ready(ready) => {
            let mut redis = event_context.redis.get().await;
            redis
                .set("katze_current_user", ready.user.id.to_string())
                .await?;
        }
        Event::MessageCreate(message) if message.content.starts_with("katze ") => {
            let content = message.content.to_owned();
            let mut content = content.split(' ').skip(1);

            // read the next word from the message as the command name
            if let Some(command) = content.next() {
                // create a MessageContext for convenience,
                // taking ownership of everything from event_context
                let context = MessageContext {
                    cache: event_context.cache,
                    http: event_context.http,
                    pool: event_context.pool,
                    redis: event_context.redis,
                    standby: event_context.standby,
                    message: message,
                    content: content.collect::<Vec<_>>().join(&" "),
                };

                // execute the command
                let result = match command {
                    "avatar" => commands::avatar(&context).await,
                    "choose" => commands::choose(&context).await,
                    "invite" => commands::invite(&context).await,
                    "movie" => commands::movie(&context).await,
                    "owo" => commands::owo(&context).await,
                    "ping" => commands::ping(&context).await,
                    "shuffle" => commands::shuffle(&context).await,
                    _ => Ok(Response::None),
                };

                match result {
                    Ok(response) => log_response(&context, &response, command),
                    Err(why) => log_error(&context, why, command),
                }
            }
        }
        Event::GuildCreate(guild) => {
            log::info!("GUILD_CREATE {}:{}", guild.id, guild.name);
            sqlx::query!(
                "INSERT INTO guilds (id, name) VALUES ($1, $2)
                ON CONFLICT (id) DO UPDATE SET name = $2;",
                guild.id.to_string(),
                guild.name
            )
            .execute(&event_context.pool)
            .await?;
        }
        Event::ReactionAdd(reaction) => {
            let context = ReactionContext {
                cache: event_context.cache,
                http: event_context.http,
                pool: event_context.pool,
                redis: event_context.redis,
                // deref the Box, and then take ownership of the Reaction
                reaction: (*reaction).0,
            };

            let mut redis = context.redis.get().await;

            // check if our id is the same as the event
            let current_id = redis.get("katze_current_user").await?.unwrap();
            if current_id == context.reaction.user_id.to_string().into_bytes() {
                return Ok(());
            }

            // scan for a menu
            let pattern = format!(
                "reaction_menu:{}:{}:*",
                context.reaction.channel_id, context.reaction.message_id
            );

            let keys = redis.scan().pattern(&pattern).run();
            let keys: Vec<Vec<u8>> = keys.collect().await;

            if keys.len() == 0 {
                ()
            }

            let key = keys.into_iter().next().unwrap();
            let key = String::from_utf8(key)?;
            let key = key.split(":").last().unwrap();

            context
                .http
                .delete_reaction(
                    context.reaction.channel_id,
                    context.reaction.message_id,
                    context.reaction.emoji.clone(),
                    context.reaction.user_id,
                )
                .await?;

            reactions::handle_event(&context, key).await?;
        }
        _ => {}
    }

    Ok(())
}
