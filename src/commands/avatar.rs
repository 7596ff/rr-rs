use anyhow::Result;

use crate::{
    model::{MessageContext, Response},
    util,
};

pub async fn avatar(context: &MessageContext) -> Result<Response> {
    let found_user = util::find_member(&context, &context.content).await?;

    let user = match found_user {
        Some(user) => user,
        None => context.message.author.clone(),
    };

    if let Some(avatar) = user.avatar {
        let content = format!(
            "https://cdn.discordapp.com/avatars/{}/{}?size=2048",
            user.id, avatar
        );

        let reply = context.reply(content).await;
        return Ok(util::construct_response(reply));
    }

    Ok(Response::None)
}
