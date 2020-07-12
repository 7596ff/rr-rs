use std::collections::HashMap;

use anyhow::Result;
use lazy_static::lazy_static;

use crate::{
    checks::CheckError,
    commands, logger,
    model::{MessageContext, Response},
};

lazy_static! {
    static ref ALIAS: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();

        m.insert("avatar", "avatar");
        m.insert("change-avatar", "change-avatar");
        m.insert("choose", "choose");
        m.insert("invite", "invite");
        m.insert("movie", "movie");
        m.insert("owo", "owo");
        m.insert("ping", "ping");
        m.insert("shuffle", "shuffle");

        m
    };
}

pub async fn handle(mut context: MessageContext) -> Result<()> {
    if let Some(prefix) = context.next() {
        if prefix != "katze" {
            return Ok(());
        }
    }

    // read the next word from the message as the command name
    if let Some(command) = context.next() {
        if let Some(command) = ALIAS.get(command.as_str()) {
            // execute the command
            let result = match *command {
                "avatar" => commands::avatar(&mut context).await,
                "change-avatar" => commands::change_avatar(&context).await,
                "choose" => commands::choose(&context).await,
                "invite" => commands::invite(&context).await,
                "movie" => commands::movie(&mut context).await,
                "owo" => commands::owo(&context).await,
                "ping" => commands::ping(&context).await,
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
        };
    }

    Ok(())
}
