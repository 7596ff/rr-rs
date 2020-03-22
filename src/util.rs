use serenity::{
    client::Context,
    model::{channel::Message, user::User},
    Error,
};
use log::{error, info};

fn handle_http_error(msg: &Message, command: &str, why: serenity::Error) {
    if let serenity::Error::Http(response) = why {
        if let serenity::http::HttpError::UnsuccessfulRequest(response) = *response {
            error!(
                "channel:{} timestamp:{} command:{} {} {}",
                msg.channel_id,
                msg.timestamp.to_rfc3339(),
                response.status_code,
                response.error.message,
                command,
            );
        }
    }
}

pub fn find_member(ctx: &Context, msg: &Message, search_str: &str) -> Option<User> {
    if msg.mentions.first().is_some() {
        return msg.mentions.first().cloned();
    }

    let guild_lock = msg.channel(&ctx.cache)?.guild()?;
    let guild = guild_lock.read();
    let members = guild.members(&ctx.cache).ok()?;

    let found = members
        .iter()
        .find(|&member| member.display_name().into_owned() == search_str.to_string());

    if found.is_some() {
        let user = found.unwrap().user.read();
        return Some(user.clone());
    } else {
        return Some(msg.author.clone());
    }
}

pub fn handle_sent_message(msg: &Message, result: Result<Message, Error>, command: &str) {
    match result {
        Ok(sent) => {
            info!(
                "channel:{} timestamp:{} command:{}",
                sent.channel_id,
                sent.timestamp.to_rfc3339(),
                command,
            );
        }
        Err(why) => handle_http_error(&msg, &command, why),
    }
}
