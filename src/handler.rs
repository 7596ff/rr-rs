use log::{error, info};
use serenity::{
    client::{Context, EventHandler},
    model::channel::Message,
    Error as SerenityError,
};

use crate::commands;
use crate::error::Error;

pub struct Handler;

impl EventHandler for Handler {
    fn message(&self, ctx: Context, msg: Message) {
        let content = msg.content.to_owned();
        let mut content = content.split(' ');

        // prefix
        if content.next() != Some(&"katze") {
            return;
        }

        if let Some(command) = content.next() {
            let result = match command {
                "ping" => commands::ping(&ctx, &msg),
                _ => return,
            };

            match result {
                Ok(sent) => {
                    info!(
                        "command:{} channel:{} timestamp:{}",
                        command,
                        sent.channel_id,
                        sent.timestamp.to_rfc3339()
                    );
                }
                Err(Error::SerenityMessageSendError { source }) => {
                    if let SerenityError::Http(response) = source {
                        if let serenity::http::HttpError::UnsuccessfulRequest(response) = *response
                        {
                            error!(
                                "command:{} channel:{} timestamp:{} {} {}",
                                command,
                                msg.channel_id,
                                msg.timestamp.to_rfc3339(),
                                response.status_code,
                                response.error.message
                            );
                        }
                    }
                }
                Err(_) => {}
            }
        }
    }
}
