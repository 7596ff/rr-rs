use anyhow::Result;
use twilight::model::{channel::ReactionType, gateway::payload::ReactionAdd};

use crate::model::MessageContext;

pub async fn did_you_mean(context: &MessageContext, name: &String) -> Result<bool> {
    let bystander = context
        .http
        .create_message(context.message.channel_id)
        .content(format!("Did you mean: \"{}\"?", name))?
        .await?;

    let emojis = [
        ReactionType::Unicode { name: "✅".to_string() },
        ReactionType::Unicode { name: "❎".to_string() },
    ];

    for emoji in &emojis {
        context
            .http
            .create_reaction(context.message.channel_id, bystander.id, emoji.clone())
            .await?;
    }

    let author_id = context.message.author.id;
    let reaction = context
        .standby
        .wait_for_reaction(bystander.id, move |event: &ReactionAdd| event.user_id == author_id)
        .await?;

    context.http.delete_message(bystander.channel_id, bystander.id).await?;

    Ok(reaction.emoji == emojis[0])
}
