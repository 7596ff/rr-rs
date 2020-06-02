use log::{error, info};
use tokio::stream::StreamExt;
use twilight::gateway::shard::Event;

use crate::{
    commands,
    model::{EventContext, MessageContext, Response},
};

pub async fn handle_event(event_context: EventContext) -> anyhow::Result<()> {
    match event_context.event {
        Event::MessageCreate(msg) if msg.content.starts_with("katze ") => {
            let content = msg.content.to_owned();
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
                    // deref the Box, and then take ownership of the Message
                    message: (*msg).0,
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
                    Ok(response) => match response {
                        Response::Some(reply) => {
                            // this is a reply success
                            info!(
                                "channel:{} timestamp:{} command:{}",
                                reply.channel_id.to_string(),
                                reply.timestamp,
                                command
                            );
                        }
                        Response::Err(why) => {
                            // this is a message send error
                            error!(
                                "channel:{} timestamp:{} command:{}\nerror sending message\n{:?}",
                                context.message.channel_id, context.message.timestamp, command, why
                            );
                        }
                        Response::Reaction(emoji) => {
                            context
                                .http
                                .create_reaction(
                                    context.message.channel_id,
                                    context.message.id,
                                    emoji,
                                )
                                .await?;

                            info!(
                                "channel:{} timestamp:{} command:{}",
                                context.message.channel_id, context.message.timestamp, command
                            );
                        }
                        Response::None => {}
                    },
                    Err(why) => {
                        // this is a command execution error
                        error!(
                            "channel:{} timestamp:{}\nerror processing command:{}\n{:?}",
                            context.message.channel_id, context.message.timestamp, command, why
                        );
                    }
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
        Event::ReactionAdd(_reaction) => {}
        _ => {}
    }

    Ok(())
}
