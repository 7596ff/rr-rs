use anyhow::Result;

use crate::{
    checks::CheckError,
    commands, logger,
    model::{MessageContext, Response},
};

pub async fn handle(mut context: MessageContext) -> Result<()> {
    if let Some(prefix) = context.next() {
        if prefix != "katze" {
            return Ok(());
        }
    }

    // read the next word from the message as the command name
    if let Some(command) = context.next() {
        // execute the command
        let result = match command.as_ref() {
            "avatar" => commands::avatar(&mut context).await,
            "change-avatar" => commands::change_avatar(&context).await,
            "choose" => commands::choose(&context).await,
            "invite" => commands::invite(&context).await,
            "movie" => commands::movie(&mut context).await,
            "owo" => commands::owo(&context).await,
            "ping" | "pong" => commands::ping(&context).await,
            "shuffle" => commands::shuffle(&mut context).await,
            _ => Ok(Response::None),
        };

        // if we fail a check, tell the user
        if let Err(why) = &result {
            if let Some(check_error) = why.downcast_ref::<CheckError>() {
                context.reply(format!("{}", check_error)).await?;
            }
        }

        match result {
            Ok(response) => logger::response(context, response, command.to_string()),
            Err(why) => logger::error(context, why, command.to_string()),
        }
    }

    Ok(())
}
