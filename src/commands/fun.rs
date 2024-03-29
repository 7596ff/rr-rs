use crate::model::{GenericError, MessageContext, Response};

pub async fn owo(context: &MessageContext) -> Result<Response, GenericError> {
    let grid = "O
O X X O X O X O X O X X O
X O O X X O X O X X O O X
X O O X X O X O X X O O X
X O O X X O X O X X O O X
O X X O X X X X X O X X O";

    let owo = grid
        .replace("O", "<:a:279251926409936896>")
        .replace("X", "<:a:368359173618008064>");

    let reply = context.reply(owo).await?;
    Ok(Response::Message(reply))
}
