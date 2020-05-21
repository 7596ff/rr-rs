use anyhow::Result;
use twilight::{http::Client as HttpClient, model::channel::Message};

use crate::{model::Response, util};

pub async fn ping(msg: &Message, http: &HttpClient) -> Result<Response> {
    let sent = http.create_message(msg.channel_id).content("pong!").await;

    Ok(util::construct_response(sent))

    // match msg.channel_id.say(&ctx, "pong!") {
    //     Ok(mut sent) => {
    //         let latency = sent.timestamp.timestamp_millis() - msg.timestamp.timestamp_millis();

    //         // i can't figure out a way to care about this result
    //         let _ = sent.edit(ctx, |m| {
    //             m.content(format!("🏓 Message send latency: {} ms", latency))
    //         });

    //         Some(Ok(sent))
    //     }
    //     Err(why) => Some(Err(why)),
    // }
}
