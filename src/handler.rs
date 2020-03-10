use log::{error, info};
use serenity::{
    client::{Context, EventHandler},
    model::channel::Message,
};

use crate::commands;

pub struct Handler;

fn handle_http_error(msg: &Message, command: &str, why: serenity::Error) {
    if let serenity::Error::Http(response) = why {
        if let serenity::http::HttpError::UnsuccessfulRequest(response) = *response {
            error!(
                "channel:{} timestamp:{} command:{} {} {}",
                msg.channel_id,
                msg.timestamp.to_rfc3339(),
                response.status_code,
                response.error.message,
                command,
            );
        }
    }
}

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
                        "channel:{} timestamp:{} command:{}",
                        sent.channel_id,
                        sent.timestamp.to_rfc3339(),
                        command,
                    );
                }
                Err(why) => handle_http_error(&msg, &command, why),
            }
        }
    }
}
