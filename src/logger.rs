use crate::{
    checks::CheckError,
    model::{MessageContext, Response},
};
use log::{error, info};
use twilight_http::{
    api_error::ApiError,
    error::{Error as HttpError, ErrorType as HttpErrorType},
};

pub fn response(context: &MessageContext, response: &Response, command: String) {
    match response {
        Response::Message(reply) => {
            info!(
                "channel:{} timestamp:{} command:{}",
                reply.channel_id, reply.timestamp, command
            )
        }
        Response::Reaction => info!(
            "channel:{} timestamp:{} command:{}",
            context.message.channel_id, context.message.timestamp, command
        ),
        Response::None => {}
    }
}

pub fn error(context: &MessageContext, why: &anyhow::Error, command: String) {
    if let Some(error) = why.downcast_ref::<HttpError>() {
        if let HttpErrorType::Response {
            error: ApiError::General(general),
            ..
        } = error.kind()
        {
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
    } else if let Some(why) = why.downcast_ref::<CheckError>() {
        info!(
            "channel:{} timestamp:{} command:{} {:?}",
            context.message.channel_id, context.message.timestamp, command, why
        );
    } else {
        error!(
            "channel:{} timestamp:{}\nerror processing command:{}\n{:#?}",
            context.message.channel_id, context.message.timestamp, command, why
        );
    }
}
