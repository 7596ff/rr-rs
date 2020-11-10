pub mod automation;

use anyhow::Result;

use crate::{
    checks::CheckError,
    commands, logger,
    model::{MessageContext, Response},
};

pub async fn handle(mut context: MessageContext) -> Result<()> {
    // don't process messages from bots
    if context.message.author.bot {
        return Ok(());
    }

    // run automation checks in a new task
    let auto_context = context.clone();
    tokio::spawn(async move {
        let mut autos = Vec::new();
        autos.push(("emojis", automation::emojis(&auto_context).await));
        autos.push(("vtrack", automation::vtrack(&auto_context).await));

        for (name, result) in autos.iter() {
            if let Err(why) = result {
                logger::error(&auto_context, why, name.to_string());
            }
        }
    });

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
            "emojis" => commands::util::emojis(&context).await,
            "help" => commands::util::help(&context).await,
            "invite" => commands::util::invite(&context).await,
            "list" | "ls" => commands::rotate::list(&context).await,
            "movie" => commands::movie::execute(&mut context).await,
            "owo" => commands::fun::owo(&context).await,
            "pick" => commands::rotate::pick(&mut context).await,
            "ping" | "pong" => commands::util::ping(&context).await,
            "rotate" | "rotato" | "tomato" | "potato" | "🍅" | "🥔" => {
                commands::rotate::execute(&mut context).await
            }
            "show" => commands::rotate::show(&mut context).await,
            "shuffle" => commands::util::shuffle(&mut context).await,
            "steal" => commands::util::steal(&mut context).await,
            _ => Ok(Response::None),
        };

        // if we fail a check, tell the user
        if let Err(why) = &result {
            if let Some(check_error) = why.downcast_ref::<CheckError>() {
                context.reply(format!("{}", check_error)).await?;
            }
        }

        match result {
            Ok(response) => logger::response(&context, &response, command.to_string()),
            Err(why) => logger::error(&context, &why, command.to_string()),
        }
    }

    Ok(())
}
