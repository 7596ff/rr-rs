mod message;
mod reaction;

use anyhow::Result;
use twilight::gateway::Event;

use crate::model::{EventContext, MessageContext, ReactionContext};

pub async fn event(event: Event, event_context: EventContext) -> Result<()> {
    match event {
        Event::Ready(ready) => {
            let mut redis = event_context.redis.get().await;
            redis.set("katze_current_user", ready.user.id.to_string()).await?;
            Ok(())
        }
        Event::MessageCreate(message) => {
            message::handle(MessageContext::new(event_context, message)?).await
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
            Ok(())
        }
        Event::ReactionAdd(reaction) => {
            reaction::handle(ReactionContext::new(event_context, reaction)).await
        }
        _ => Ok(()),
    }
}
