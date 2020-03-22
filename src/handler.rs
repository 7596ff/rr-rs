use serenity::{
    client::{Context, EventHandler},
    model::channel::Message,
};

use crate::commands;

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
            // join the rest of the content into a string for the commands to use
            let content = content.collect::<Vec<_>>().join(&" ");

            // and execute the command
            let _ = match command {
                "avatar" => commands::avatar(&ctx, &msg, &content),
                "ping" => commands::ping(&ctx, &msg),
                _ => return,
            };
        }
    }
}
