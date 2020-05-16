use serenity::{client::Context, model::channel::Message, Error};

pub fn ping(ctx: &Context, msg: &Message) -> Option<Result<Message, Error>> {
    match msg.channel_id.say(&ctx, "pong!") {
        Ok(mut sent) => {
            let latency = sent.timestamp.timestamp_millis() - msg.timestamp.timestamp_millis();

            // i can't figure out a way to care about this result
            let _ = sent.edit(ctx, |m| {
                m.content(format!("🏓 Message send latency: {} ms", latency))
            });

            Some(Ok(sent))
        }
        Err(why) => Some(Err(why)),
    }
}
