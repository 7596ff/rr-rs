use anyhow::Result;

use crate::{
    commands::{self, Command},
    logger,
    model::{MessageContext, Response},
};

async fn execute(command: &mut (impl Command<MessageContext> + Sync)) -> Result<Response> {
    if command.check().await? {
        command.execute().await
    } else {
        Ok(Response::None)
    }
}

pub async fn handle(mut context: MessageContext) -> Result<()> {
    if let Some(prefix) = context.next() {
        if prefix != "katze" {
            return Ok(());
        }
    }

    // read the next word from the message as the command name
    if let Some(command) = context.next() {
        let result = match command.as_ref() {
            "avatar" => execute(&mut commands::Avatar(context.clone())).await,
            "change-avatar" => execute(&mut commands::ChangeAvatar(context.clone())).await,
            "choose" => execute(&mut commands::Choose(context.clone())).await,
            "invite" => execute(&mut commands::Invite(context.clone())).await,
            _ => execute(&mut commands::NoCommand(context.clone())).await,
        };

        match result {
            Ok(response) => logger::response(context, response, command.to_string()),
            Err(why) => logger::error(context, why, command.to_string()),
        }
        //         "movie" => commands::movie(&mut context).await,
        //         "owo" => commands::owo(&context).await,
        //         "ping" => commands::ping(&context).await,
        //         "shuffle" => commands::shuffle(&mut context).await,
        //         _ => Ok(Response::None),
        //     };

        //     match result {
        //         Ok(response) => logger::response(context, response, command.to_string()),
        //         Err(why) => logger::error(context, why, command.to_string()),
        //     }
        // };
    }

    Ok(())
}
