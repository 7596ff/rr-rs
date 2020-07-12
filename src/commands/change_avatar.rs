use anyhow::Result;
use futures_util::io::AsyncReadExt;

use crate::{
    checks,
    model::{MessageContext, Response, ResponseReaction},
};

pub async fn change_avatar(context: &MessageContext) -> Result<Response> {
    checks::is_owner(&context)?;

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
