use anyhow::Result;
use async_trait::async_trait;

use crate::model::{MessageContext, Response};

pub struct Invite(pub MessageContext);

#[async_trait]
impl super::Command<MessageContext> for Invite {
    fn new(context: MessageContext) -> Self {
        Self(context)
    }

    async fn execute(self: &mut Self) -> Result<Response> {
        let content = "<https://discordapp.com/oauth2/authorize?client_id=254387001556598785&permissions=268435488&scope=bot>";

        let reply = self.0.reply(content).await?;
        Ok(Response::Message(reply))
    }
}
