use anyhow::Result;
use async_trait::async_trait;
use rand::{seq::SliceRandom, thread_rng};

use crate::model::{MessageContext, Response};

async fn _execute(context: &MessageContext) -> Result<Response> {
    let maybe_item = context.args.choose(&mut thread_rng());

    let item = match maybe_item {
        Some(item) => item,
        None => return Ok(Response::None),
    };

    let reply = context.reply(item).await?;
    Ok(Response::Message(reply))
}

pub struct Choose(pub MessageContext);

#[async_trait]
impl super::Command<MessageContext> for Choose {
    fn new(context: MessageContext) -> Self {
        Self(context)
    }

    async fn execute(self: &mut Self) -> Result<Response> {
        _execute(&self.0).await
    }
}
