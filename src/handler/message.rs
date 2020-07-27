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
            "add_image" | "pls" => commands::rotate::add_image(&context).await,
            "avatar" => commands::util::avatar(&mut context).await,
            "change-avatar" => commands::admin::change_avatar(&context).await,
            "count" => commands::rotate::count(&context).await,
            "choose" => commands::util::choose(&context).await,
            "delete" | "remove" | "rm" => commands::rotate::delete(&mut context).await,
            "invite" => commands::util::invite(&context).await,
            "list" => commands::rotate::list(&context).await,
            "movie" => commands::movie::execute(&mut context).await,
            "owo" => commands::fun::owo(&context).await,
            "pick" => commands::rotate::pick(&mut context).await,
            "ping" | "pong" => commands::util::ping(&context).await,
            "rotate" | "rotato" | "tomato" | "potato" | "🍅" | "🥔" => {
                commands::rotate::execute(&mut context).await
            }
            "show" => commands::rotate::show(&mut context).await,
            "shuffle" => commands::util::shuffle(&mut context).await,
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
