use anyhow::Result;
use chrono::DateTime;

use crate::model::{MessageContext, Response};

pub async fn ping(context: &MessageContext) -> Result<Response> {
    let sent = context.reply("pong!").await?;

    let sent_time = DateTime::parse_from_rfc3339(sent.timestamp.as_str())?;
    let message_time = DateTime::parse_from_rfc3339(context.message.timestamp.as_str())?;
    let latency = sent_time.timestamp_millis() - message_time.timestamp_millis();

    let update = context
        .http
        .update_message(context.message.channel_id, sent.id)
        .content(format!("🏓 Message send latency: {} ms", latency))?
        .await?;

    Ok(Response::Message(update))
}
