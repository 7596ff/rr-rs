use anyhow::Result;
use chrono::DateTime;

use crate::{
    model::{MessageContext, Response},
    util,
};

pub async fn ping(context: &MessageContext) -> Result<Response> {
    let sent = context
        .http
        .create_message(context.message.channel_id)
        .content("pong!")
        .await;

    match sent {
        Ok(sent) => {
            let sent_time = DateTime::parse_from_rfc3339(sent.timestamp.as_str())?;
            let message_time = DateTime::parse_from_rfc3339(context.message.timestamp.as_str())?;
            let latency = sent_time.timestamp_millis() - message_time.timestamp_millis();

            let update = context
                .http
                .update_message(context.message.channel_id, sent.id)
                .content(format!("🏓 Message send latency: {} ms", latency))
                .await;

            Ok(util::construct_response(update))
        }
        // technically a message send failure, so act like it
        Err(why) => Ok(Response::Err(why)),
    }
}
