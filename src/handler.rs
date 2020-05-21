use log::{error, info};
use twilight::gateway::shard::Event;

use crate::{
    commands,
    model::{EventContext, MessageContext, Response},
};

pub async fn handle_event(event_context: EventContext) -> anyhow::Result<()> {
    match event_context.event {
        Event::MessageCreate(msg) if msg.content.starts_with("katze") => {
            let content = msg.content.to_owned();
            let mut content = content.split(' ');
            let _ = content.next();

            // read the next word from the message as the command name
            if let Some(command) = content.next() {
                // create a message context for convenience
                let message_context = MessageContext {
                    cache: event_context.cache,
                    http: event_context.http,
                    // deref the Box, and then take ownership of the Message
                    message: (*msg).0,
                    content: content.collect::<Vec<_>>().join(&" "),
                };

                // execute the command
                let result = match command {
                    "avatar" => commands::avatar(&message_context).await,
                    "ping" => commands::ping(&message_context).await,
                    // "owo" => commands::owo(&ctx, &msg),
                    _ => Ok(Response::None),
                };

                match result {
                    Ok(response) => match response {
                        Response::Some(reply) => {
                            // this is a reply success
                            info!(
                                "channel:{} timestamp:{} command:{}",
                                reply.channel_id.to_string(),
                                reply.timestamp,
                                command
                            );
                        }
                        Response::Err(why) => {
                            // this is a message send error
                            error!(
                                "channel:{} timestamp:{} command:{}\nerror sending message\n{:?}",
                                message_context.message.channel_id,
                                message_context.message.timestamp,
                                command,
                                why
                            );
                        }
                        Response::None => {}
                    },
                    Err(why) => {
                        // this is a command execution error
                        error!(
                            "channel:{} timestamp:{}\nerror processing command:{}\n{:?}",
                            message_context.message.channel_id,
                            message_context.message.timestamp,
                            command,
                            why
                        );
                    }
                }
            }
        }
        _ => {}
    }

    Ok(())
}
