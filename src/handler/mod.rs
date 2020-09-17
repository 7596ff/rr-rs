mod message;
mod reaction;

use anyhow::Result;
use twilight_gateway::Event;

use crate::model::{Context, MessageContext, ReactionContext};

pub async fn event(event: Event, context: Context) -> Result<()> {
    match event {
        Event::Ready(ready) => {
            let mut redis = context.redis.get().await;
            redis.set("katze_current_user", ready.user.id.to_string()).await?;
            Ok(())
        }
        Event::MessageCreate(message) => {
            message::handle(MessageContext::new(context, message)?).await
        }
        Event::GuildCreate(guild) => {
            log::info!("GUILD_CREATE {}:{}", guild.id, guild.name);

            sqlx::query!(
                "INSERT INTO guilds (id, name) VALUES ($1, $2)
                ON CONFLICT (id) DO UPDATE SET name = $2;",
                guild.id.to_string(),
                guild.name
            )
            .execute(&context.pool)
            .await?;

            sqlx::query!(
                "INSERT INTO settings (guild_id) VALUES ($1)
                ON CONFLICT (guild_id) DO NOTHING;",
                guild.id.to_string(),
            )
            .execute(&context.pool)
            .await?;

            Ok(())
        }
        Event::ReactionAdd(reaction) => {
            reaction::handle(ReactionContext::new(context, reaction)).await
        }
        _ => Ok(()),
    }
}
