use serenity::{client::Context, model::channel::Message, Error};

use crate::util;

pub fn avatar(ctx: &Context, msg: &Message, search_str: &str) -> Option<Result<Message, Error>> {
    let found_user = util::find_member(&ctx, &msg, &search_str);

    match found_user {
        Some(user) => match user.avatar_url() {
            Some(url) => Some(msg.channel_id.say(&ctx, &url)),
            None => Some(msg.channel_id.say(&ctx, &user.default_avatar_url())),
        },
        None => None,
    }
}
