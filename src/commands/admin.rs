use crate::{
    checks,
    model::{MessageContext, Response, ResponseReaction},
};
use anyhow::Result;
use hyper::{
    body::{self, Body},
    Request,
};

pub async fn change_avatar(context: &MessageContext) -> Result<Response> {
    checks::is_owner(&context)?;

    let url = if context.message.attachments.is_empty() {
        context.args.join(" ")
    } else {
        context.message.attachments.first().unwrap().url.clone()
    };

    let request = Request::get(url).body(Body::empty())?;
    let mut response = context.hyper.request(request).await?;
    let buffer = body::to_bytes(response.body_mut()).await?;

    context
        .http
        .update_current_user()
        .username("rr-rs")?
        .avatar(format!("data:image/png;base64,{}", base64::encode(buffer)))
        .await?;

    context.react(ResponseReaction::Success.value()).await?;
    Ok(Response::Reaction)
}
