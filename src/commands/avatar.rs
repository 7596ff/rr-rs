use anyhow::Result;
use twilight::{http::Client as HttpClient, model::channel::Message};

use crate::{model::Response, util};

pub async fn avatar(msg: &Message, http: &HttpClient, content: &str) -> Result<Response> {
    let found_user = util::find_member(&ctx, &msg, &content);

    match found_user {
        Some(user) => match user.avatar_url() {
            Some(url) => Some(msg.channel_id.say(&ctx, &url)),
            None => Some(msg.channel_id.say(&ctx, &user.default_avatar_url())),
        },
        None => None,
    }
}
