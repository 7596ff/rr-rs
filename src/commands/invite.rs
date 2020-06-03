use anyhow::Result;

use crate::{
    model::{MessageContext, Response},
    util,
};

pub async fn invite(context: &MessageContext) -> Result<Response> {
    let content = "<https://discordapp.com/oauth2/authorize?client_id=254387001556598785&permissions=268435488&scope=bot>";

    Ok(util::construct_response(
        context
            .http
            .create_message(context.message.channel_id)
            .content(content)?
            .await,
    ))
}
