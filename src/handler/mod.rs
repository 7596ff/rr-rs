mod message;

use crate::model::{BaseContext, GenericError, MessageContext};
use chrono::Utc;
use twilight_gateway::Event;
use twilight_model::channel::ReactionType;

pub async fn event(event: Event, context: BaseContext) -> Result<(), GenericError> {
    let now = Utc::now();

    match event {
        Event::Ready(ready) => {
            let mut redis = context.redis().get().await;
            redis
                .set("katze_current_user", ready.user.id.to_string())
                .await?;
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
                guild.name,
            )
            .execute(context.postgres())
            .await?;

            sqlx::query!(
                "INSERT INTO settings (guild_id) VALUES ($1)
                ON CONFLICT (guild_id) DO NOTHING;",
                guild.id.to_string(),
            )
            .execute(context.postgres())
            .await?;

            Ok(())
        }
        Event::ReactionAdd(reaction) => {
            // store reaction in counts before we do anything
            // also ignore if the reaction comes from a bot
            if let Some(member) = &reaction.member {
                if member.user.bot {
                    return Ok(());
                }

                if let ReactionType::Custom { id, .. } = &reaction.emoji {
                    sqlx::query!(
                        "INSERT INTO emojis
                        (datetime, guild_id, message_id, member_id, emoji_id, reaction)
                        VALUES ($1, $2, $3, $4, $5, true)",
                        now.timestamp(),
                        reaction.guild_id.unwrap().to_string(),
                        reaction.message_id.to_string(),
                        reaction.user_id.to_string(),
                        id.to_string(),
                    )
                    .execute(context.postgres())
                    .await?;
                }
            }

            Ok(())
        }
        Event::ReactionRemove(reaction) => {
            // this operation is safe, even if the user is a bot, because the delete operation will
            // delete 0 rows.
            if let ReactionType::Custom { id, .. } = &reaction.emoji {
                sqlx::query!(
                    "DELETE FROM emojis WHERE
                    (message_id = $1 AND member_id = $2 AND emoji_id = $3 AND reaction = true);",
                    reaction.message_id.to_string(),
                    reaction.user_id.to_string(),
                    id.to_string(),
                )
                .execute(context.postgres())
                .await?;
            }

            Ok(())
        }
        Event::ReactionRemoveAll(data) => {
            sqlx::query!(
                "DELETE FROM emojis WHERE
                (guild_id = $1 AND message_id = $2 AND reaction = true);",
                data.guild_id.unwrap().to_string(),
                data.message_id.to_string(),
            )
            .execute(context.postgres())
            .await?;

            Ok(())
        }
        Event::ReactionRemoveEmoji(data) => {
            if let ReactionType::Custom { id, .. } = &data.emoji {
                sqlx::query!(
                    "DELETE FROM emojis WHERE
                    (guild_id = $1 AND message_id = $2 AND emoji_id = $3 AND reaction = true);",
                    data.guild_id.to_string(),
                    data.message_id.to_string(),
                    id.to_string(),
                )
                .execute(context.postgres())
                .await?;
            }

            Ok(())
        }
        _ => Ok(()),
    }
}
