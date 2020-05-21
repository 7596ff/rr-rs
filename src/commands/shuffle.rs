use anyhow::Result;
use rand::{seq::SliceRandom, thread_rng};

use crate::{
    model::{MessageContext, Response},
    util,
};

pub async fn shuffle(context: &MessageContext) -> Result<Response> {
    let mut items: Vec<&str> = context.content.split(" ").collect();
    items.shuffle(&mut thread_rng());
    let mut items = items.iter();
    let mut counter: i32 = 0;
    let mut content = String::new();

    while let Some(item) = items.next() {
        counter += 1;
        content.push_str(&format!("`{}` {}\n", counter, item));
    }

    Ok(util::construct_response(
        context
            .http
            .create_message(context.message.channel_id)
            .content(content)
            .await,
    ))
}
