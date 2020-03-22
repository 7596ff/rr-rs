use crate::util;
use serenity::{client::Context, model::channel::Message, Error};

pub fn avatar(ctx: &Context, msg: &Message, search_str: &str) -> Result<(), Error> {
    let found_user = util::find_member(&ctx, &msg, &search_str);

    match found_user {
        Some(user) => {
            let sent = match user.avatar_url() {
                Some(url) => msg.channel_id.say(&ctx, &url),
                None => msg.channel_id.say(&ctx, &user.default_avatar_url()),
            };
            util::handle_sent_message(&msg, sent, "avatar");
        },
        None => ()
    }

    Ok(())
}
