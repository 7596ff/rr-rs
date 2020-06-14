use anyhow::Result;
use rand::{seq::SliceRandom, thread_rng};

use crate::model::{MessageContext, Response};

pub async fn choose(context: &MessageContext) -> Result<Response> {
    let items: Vec<&str> = context.content.split(" ").collect();
    let maybe_item = items.choose(&mut thread_rng());

    let item = match maybe_item {
        Some(item) => item,
        None => return Ok(Response::None),
    };

    let reply = context.reply(*item).await;
    Ok(Response::Message(reply))
}
