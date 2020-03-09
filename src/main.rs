use dotenv;
use pretty_env_logger;
use serenity::Client;
use snafu::ResultExt;

mod commands;
mod error;
mod handler;
use crate::error::*;
use crate::handler::Handler;

fn main() -> Result<(), Error> {
    // load dotenv and logger
    dotenv::dotenv().ok();
    pretty_env_logger::init();

    // create and start bot
    let mut client = Client::new(dotenv::var("TOKEN").unwrap().as_str(), Handler)
        .context(SerenityClientInvalidToken)?;

    client
        .start_autosharded()
        .context(SerenityClientShardBootFailure)?;

    Ok(())
}
