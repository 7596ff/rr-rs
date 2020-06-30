use anyhow::Result;
use futures_util::io::AsyncReadExt;

use crate::model::{MessageContext, Response};

pub async fn change_avatar(context: &MessageContext) -> Result<Response> {
    if dotenv::var("OWNER")?.parse::<u64>()? != context.message.author.id.0 {
        let reply = context.reply("You are not the owner.").await?;
        return Ok(Response::Message(reply));
    }

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

    context.react("✅").await?;
    Ok(Response::Reaction)
}
