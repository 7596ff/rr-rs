use anyhow::Result;

use crate::model::{MessageContext, Response};

pub async fn avatar(context: &mut MessageContext) -> Result<Response> {
    let found_user = context.find_member().await?;

    let user = match found_user {
        Some(user) => user,
        None => context.message.author.clone(),
    };

    if let Some(avatar) = user.avatar {
        let content =
            format!("https://cdn.discordapp.com/avatars/{}/{}?size=2048", user.id, avatar);

        let reply = context.reply(content).await?;
        return Ok(Response::Message(reply));
    }

    Ok(Response::None)
}
