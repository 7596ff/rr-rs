use log::{error, info};
use twilight::{gateway::shard::Event, http::Client as HttpClient};

use crate::{
    commands,
    model::{Context, Response},
};

pub async fn handle_event(event: (u64, Event), http: HttpClient) -> anyhow::Result<()> {
    match event {
        (_, Event::MessageCreate(msg)) if msg.content.starts_with("katze") => {
            let content = msg.content.to_owned();
            let mut content = content.split(' ');
            let _ = content.next();

            // read the next word from the message as the command name
            if let Some(command) = content.next() {
                // join the rest of the content into a string for the commands to use
                let content = content.collect::<Vec<_>>().join(&" ");

                // execute the command
                let result = match command {
                    // "avatar" => commands::avatar(&msg, &http, &content).await,
                    "ping" => commands::ping(&msg, &http).await,
                    // "owo" => commands::owo(&ctx, &msg),
                    _ => Ok(Response::None),
                };

                match result {
                    Ok(response) => match response {
                        Response::Some(reply) => {
                            info!(
                                "channel:{} timestamp:{} command:{}",
                                reply.channel_id.to_string(),
                                reply.timestamp,
                                command
                            );
                        }
                        // TODO: handle errors better
                        // this is a message send error
                        Response::Err(why) => error!("{:?}", why),
                        Response::None => {}
                    }
                    Err(why) => {
                        // this is a command execution error
                    }
                }
            }
        }
        _ => {}
    }

    Ok(())
}
