use anyhow::Result;
use tokio::stream::StreamExt;

use crate::{model::ReactionContext, reactions};

pub async fn handle(context: ReactionContext) -> Result<()> {
    let mut redis = context.redis.get().await;

    // check if our id is the same as the event
    let current_id = redis.get("katze_current_user").await?.unwrap();
    if current_id == context.reaction.user_id.to_string().into_bytes() {
        return Ok(());
    }

    // scan for a menu
    let pattern =
        format!("reaction_menu:{}:{}:*", context.reaction.channel_id, context.reaction.message_id);

    let keys = redis.scan().pattern(&pattern).run();
    let keys: Vec<Vec<u8>> = keys.collect().await;

    if keys.is_empty() {
        return Ok(());
    }

    let key = keys.into_iter().next().unwrap();
    let key = String::from_utf8(key)?;
    let key = key.split(':').last().unwrap();

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

    Ok(())
}
