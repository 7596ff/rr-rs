use serenity::{
    client::{Context, EventHandler},
    model::channel::Message,
};

use crate::{commands, util};

pub struct Handler;

impl EventHandler for Handler {
    fn message(&self, ctx: Context, msg: Message) {
        let content = msg.content.to_owned();
        let mut content = content.split(' ');

        // prefix
        if content.next() != Some(&"katze") {
            return;
        }

        // if we can't respond with a message,
        // don't even bother processing the command
        if let Some(channel) = msg.channel(&ctx.cache) {
            if let Some(guild_channel_lock) = channel.guild() {
                let current_id = &ctx.cache.read().user.id;
                if let Ok(permissions) = guild_channel_lock
                    .read()
                    .permissions_for_user(&ctx.cache, current_id)
                {
                    if !permissions.send_messages() {
                        return;
                    }
                }
            }
        }

        // read the next word from the message as the command name
        if let Some(command) = content.next() {
            // join the rest of the content into a string for the commands to use
            let content = content.collect::<Vec<_>>().join(&" ");

            // execute the command
            let result = match command {
                "avatar" => commands::avatar(&ctx, &msg, &content),
                "ping" => commands::ping(&ctx, &msg),
                "owo" => commands::owo(&ctx, &msg),
                _ => None,
            };

            // sometimes a command will not reply with a message,
            // so only look for errors if it did
            if let Some(reply) = result {
                util::handle_sent_message(&msg, reply, command)
            }
        }
    }
}
