use crate::{
    checks::CheckError,
    commands, logger,
    model::{MessageContext, Response},
    table::Setting,
};
use anyhow::Result;
use chrono::{DateTime, TimeZone, Utc};
use chrono_humanize::{Accuracy, HumanTime, Tense};
use lazy_static::lazy_static;
use regex::Regex;
use twilight_mention::Mention;

lazy_static! {
    static ref E: Regex = Regex::new(r"<a?:[a-zA-Z1-9-_]{2,}:(?P<id>\d{17,21})>").unwrap();
    static ref V: Regex = Regex::new(r"\b[Vv][Oo][Rr][Ee]").unwrap();
}

const THIRTY_MINUTES: i64 = 1800000;

async fn emojis(context: &MessageContext) -> Result<Response> {
    let now = Utc::now();

    let ids = E
        .captures_iter(context.message.content.as_ref())
        .map(|c| c["id"].to_string())
        .collect::<Vec<String>>();

    // TODO: figure out pipelining here
    for id in ids {
        sqlx::query!(
            "INSERT INTO emojis (datetime, guild_id, message_id, member_id, emoji_id)
            VALUES ($1, $2, $3, $4, $5);",
            now.timestamp(),
            context.message.guild_id.unwrap().to_string(),
            context.message.id.to_string(),
            context.message.author.id.to_string(),
            id.clone(),
        )
        .execute(&context.postgres)
        .await?;
    }

    Ok(Response::None)
}

async fn vtrack(context: &MessageContext) -> Result<Response> {
    let guild_id = context.message.guild_id.unwrap().to_string();

    if V.is_match(context.message.content.as_ref()) {
        let setting =
            Setting::query(context.postgres.clone(), context.message.guild_id.unwrap()).await?;

        if !setting.vtrack {
            return Ok(Response::None);
        }

        // get the last time it was used, or 0 otherwise
        let mut redis = context.redis.get().await;
        let reply = redis.hget("katze:vore", &guild_id).await?;
        let reply = reply.unwrap_or_else(|| b"0".to_owned().to_vec());

        // determine the difference. use naive utc because we know both times are utc, and we only
        // care about the resultant duration
        let last_stamp = String::from_utf8(reply)?;
        let last_stamp = last_stamp.parse::<i64>()?;
        let last_stamp = Utc.timestamp(last_stamp, 0).naive_utc();

        let message_stamp = context.message.timestamp.as_ref();
        let message_stamp = DateTime::parse_from_rfc3339(message_stamp)?;
        let message_stamp = message_stamp.naive_utc();

        let difference = message_stamp - last_stamp;

        // set the latest time
        redis
            .hset(
                "katze:vore",
                &guild_id,
                &message_stamp.timestamp().to_string(),
            )
            .await?;

        // determine if we should send a message
        if difference.num_milliseconds() < THIRTY_MINUTES {
            return Ok(Response::None);
        }

        // send the message
        let content = format!(
            "{} has broken the silence and said the cursed word.
This server has gone {} since the last infraction.",
            context.message.author.mention(),
            HumanTime::from(difference).to_text_en(Accuracy::Precise, Tense::Present),
        );

        let reply = context.reply(content).await?;

        return Ok(Response::Message(reply));
    }

    Ok(Response::None)
}

pub async fn handle(mut context: MessageContext) -> Result<()> {
    // don't process messages from bots
    if context.message.author.bot {
        return Ok(());
    }

    // run automation checks in a new task
    let auto_context = context.clone();
    tokio::spawn(async move {
        #[allow(clippy::eval_order_dependence)]
        let autos = vec![
            ("emojis", emojis(&auto_context).await),
            ("vtrack", vtrack(&auto_context).await),
        ];

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
            "roleme" => commands::roleme::execute(&mut context).await,
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
