use tokio::stream::StreamExt;
use twilight::gateway::Event;

use crate::{
    commands, logger,
    model::{EventContext, MessageContext, ReactionContext, Response},
    reactions,
};

pub async fn handle_event(event: Event, event_context: EventContext) -> anyhow::Result<()> {
    match event {
        Event::Ready(ready) => {
            let mut redis = event_context.redis.get().await;
            redis.set("katze_current_user", ready.user.id.to_string()).await?;
        }
        Event::MessageCreate(message) => {
            let mut context = MessageContext::new(event_context, message)?;

            if let Some(prefix) = context.next() {
                if prefix != "katze" {
                    return Ok(());
                }
            }

            // read the next word from the message as the command name
            if let Some(command) = context.next() {
                // execute the command
                let result = match command.as_str() {
                    "avatar" => commands::avatar(&mut context).await,
                    "choose" => commands::choose(&mut context).await,
                    "invite" => commands::invite(&mut context).await,
                    "movie" => commands::movie(&mut context).await,
                    "owo" => commands::owo(&mut context).await,
                    "ping" => commands::ping(&mut context).await,
                    "shuffle" => commands::shuffle(&mut context).await,
                    _ => Ok(Response::None),
                };

                match result {
                    Ok(response) => logger::response(context, response, command),
                    Err(why) => logger::error(context, why, command),
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
            let context = ReactionContext::new(event_context, reaction);
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
