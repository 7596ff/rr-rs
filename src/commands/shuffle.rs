use anyhow::Result;
use rand::{seq::SliceRandom, thread_rng};
use std::fmt::Write;

use crate::model::{MessageContext, Response};

pub async fn shuffle(context: &mut MessageContext) -> Result<Response> {
    context.args.shuffle(&mut thread_rng());
    let mut counter: i32 = 0;
    let mut content = String::new();

    for item in &context.args {
        counter += 1;
        writeln!(content, "`{}` {}", counter, item)?;
    }

    let reply = context.reply(content).await?;
    Ok(Response::Message(reply))
}
