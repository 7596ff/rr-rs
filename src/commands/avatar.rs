use anyhow::Result;
use async_trait::async_trait;

use crate::model::{MessageContext, Response};

async fn _execute(context: &MessageContext) -> Result<Response> {
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

pub struct Avatar(pub MessageContext);

#[async_trait]
impl super::Command<MessageContext> for Avatar {
    fn new(context: MessageContext) -> Self {
        Self(context)
    }

    async fn execute(self: &mut Self) -> Result<Response> {
        _execute(&self.0).await
    }
}
