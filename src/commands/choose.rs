use anyhow::Result;
use rand::{seq::SliceRandom, thread_rng};

use crate::{
    model::{MessageContext, Response},
    util,
};

pub async fn choose(context: &MessageContext) -> Result<Response> {
    let items: Vec<&str> = context.content.split(" ").collect();
    let maybe_item = items.choose(&mut thread_rng());

    if let Some(item) = maybe_item {
        return Ok(util::construct_response(
            context
                .http
                .create_message(context.message.channel_id)
                .content(*item)
                .await,
        ));
    } else {
        return Ok(Response::None);
    }
}
