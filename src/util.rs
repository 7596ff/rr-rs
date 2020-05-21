use twilight::{http::error::Result as HttpResult, model::channel::Message};

use crate::model::Response;

// pub fn find_member(ctx: &Context, msg: &Message, search_str: &str) -> Option<User> {
//     if msg.mentions.first().is_some() {
//         return msg.mentions.first().cloned();
//     }
//
//     let guild_lock = msg.channel(&ctx.cache)?.guild()?;
//     let guild = guild_lock.read();
//     let members = guild.members(&ctx.cache).ok()?;
//
//     let found = members
//         .iter()
//         .find(|&member| member.display_name().into_owned() == search_str.to_string());
//
//     if found.is_some() {
//         let user = found.unwrap().user.read();
//         return Some(user.clone());
//     } else {
//         return Some(msg.author.clone());
//     }
// }

pub fn construct_response(sent: HttpResult<Message>) -> Response {
    match sent {
        Ok(msg) => Response::Some(msg),
        Err(why) => Response::Err(why),
    }
}
