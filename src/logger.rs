use log::{error, info};
use twilight::http::{api_error::ApiError, error::Error as HttpError};

use crate::model::{MessageContext, Response};

pub fn response(context: MessageContext, response: Response, command: String) {
    match response {
        Response::Message(reply) => {
            info!("channel:{} timestamp:{} command:{}", reply.channel_id, reply.timestamp, command)
        }
        Response::Reaction => info!(
            "channel:{} timestamp:{} command:{}",
            context.message.channel_id, context.message.timestamp, command
        ),
        Response::None => {}
    }
}

pub fn error(context: MessageContext, why: anyhow::Error, command: String) {
    if let Some(HttpError::Response { error, .. }) = why.downcast_ref::<HttpError>() {
        if let ApiError::General(general) = error {
            error!(
                "channel:{} timestamp:{} command:{}\n{} {}",
                context.message.channel_id,
                context.message.timestamp,
                command,
                general.code,
                general.message,
            );
        }
    } else if let Some(shellwords::MismatchedQuotes) =
        why.downcast_ref::<shellwords::MismatchedQuotes>()
    {
        error!(
            "channel:{} timestamp:{} command:{} mismatched quotes",
            context.message.channel_id, context.message.timestamp, command
        );
    } else {
        error!(
            "channel:{} timestamp:{}\nerror processing command:{}\n{:?}",
            context.message.channel_id, context.message.timestamp, command, why
        );
    }
}
