use serenity::{client::Context, model::channel::Message, Error};

pub fn ping(ctx: &Context, msg: &Message) -> Result<Message, Error> {
    match msg.channel_id.say(&ctx, "pong!") {
        Ok(mut sent) => {
            let latency = sent.timestamp.timestamp_millis() - msg.timestamp.timestamp_millis();
            sent.edit(ctx, |m| {
                m.content(format!("🏓 Message send latency: {} ms", latency))
            })?;
            Ok(sent)
        }
        Err(why) => Err(why),
    }
}
