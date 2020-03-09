use serenity::{client::Context, model::channel::Message};
use snafu::ResultExt;

use crate::error::*;

pub fn ping(ctx: &Context, msg: &Message) -> Result<Message, Error> {
    match msg.channel_id.say(&ctx, "pong!") {
        Ok(mut sent) => {
            let latency = sent.timestamp.timestamp_millis() - msg.timestamp.timestamp_millis();
            sent.edit(ctx, |m| {
                m.content(format!("🏓 Message send latency: {} ms", latency))
            })
            .context(SerenityMessageSendError)?;
            Ok(sent)
        }
        Err(why) => Err(why).context(SerenityMessageSendError),
    }
}
