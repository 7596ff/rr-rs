use anyhow::Result;
use rand::{seq::SliceRandom, thread_rng};
use std::fmt::Write;

use crate::model::{MessageContext, Response};

pub async fn shuffle(context: &mut MessageContext) -> Result<Response> {
    context.args.shuffle(&mut thread_rng());
    let mut content = String::new();

    for (counter, item) in context.args.iter().enumerate() {
        writeln!(content, "`{}` {}", counter, item)?;
    }

    let reply = context.reply(content).await?;
    Ok(Response::Message(reply))
}
