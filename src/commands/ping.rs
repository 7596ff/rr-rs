use serenity::{client::Context, model::channel::Message, Error};
use crate::util;

pub fn ping(ctx: &Context, msg: &Message) -> Result<(), Error> {
    match msg.channel_id.say(&ctx, "pong!") {
        Ok(mut sent) => {
            let latency = sent.timestamp.timestamp_millis() - msg.timestamp.timestamp_millis();

            let result = sent.edit(ctx, |m| {
                m.content(format!("🏓 Message send latency: {} ms", latency))
            });

            // strange matching based on how serenity handles edited messages
            // will be cleaner when i move to dawn
            match result {
                Ok(()) => util::handle_sent_message(&msg, Ok(sent), "ping"),
                Err(why) => util::handle_sent_message(&msg, Err(why), "ping"),
            }
        },
        Err(why) => util::handle_sent_message(&msg, Err(why), "ping"),
    }

    Ok(())
}
