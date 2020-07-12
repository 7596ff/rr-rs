use anyhow::Result;
use async_trait::async_trait;
use futures_util::io::AsyncReadExt;

use crate::model::{MessageContext, Response, ResponseReaction};

async fn _execute(context: &MessageContext) -> Result<Response> {
    let url = if context.message.attachments.is_empty() {
        context.args.join(" ")
    } else {
        context.message.attachments.first().unwrap().url.clone()
    };

    let mut resp = isahc::get_async(url).await?;
    let mut buffer: Vec<u8> = Vec::new();
    resp.body_mut().read_to_end(&mut buffer).await?;

    context
        .http
        .update_current_user()
        .username("rr-rs")?
        .avatar(format!("data:image/png;base64,{}", base64::encode(buffer)))
        .await?;

    context.react(ResponseReaction::Success.value()).await?;
    Ok(Response::Reaction)
}

pub struct ChangeAvatar(pub MessageContext);

#[async_trait]
impl super::Command<MessageContext> for ChangeAvatar {
    fn new(context: MessageContext) -> Self {
        Self(context)
    }

    async fn check(self: &Self) -> Result<bool> {
        Ok(dotenv::var("OWNER")?.parse::<u64>()? != self.0.message.author.id.0)
    }

    async fn execute(self: &mut Self) -> Result<Response> {
        _execute(&self.0).await
    }
}
