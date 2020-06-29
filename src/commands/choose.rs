use anyhow::Result;
use rand::{seq::SliceRandom, thread_rng};

use crate::model::{MessageContext, Response};

pub async fn choose(context: &MessageContext) -> Result<Response> {
    let maybe_item = context.args.choose(&mut thread_rng());

    let item = match maybe_item {
        Some(item) => item,
        None => return Ok(Response::None),
    };

    let reply = context.reply(item).await?;
    Ok(Response::Message(reply))
}
