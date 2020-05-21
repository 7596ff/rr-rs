pub fn owo(ctx: &Context, msg: &Message) -> Option<Result<Message, Error>> {
    let grid = "O
O X X O X O X O X O X X O
X O O X X O X O X X O O X
X O O X X O X O X X O O X
X O O X X O X O X X O O X
O X X O X X X X X O X X O";

    let owo = grid
        .replace("O", "<:a:279251926409936896>")
        .replace("X", "<:a:368359173618008064>");

    Some(msg.channel_id.say(&ctx, &owo))
}
