use anyhow::{Context, Error};
use dotenv;
use pretty_env_logger;
use serenity::Client;

mod commands;
mod handler;
use crate::handler::Handler;

fn main() -> Result<(), Error> {
    // load dotenv and logger
    dotenv::dotenv().ok();
    pretty_env_logger::init();

    // create and start bot
    let mut client = Client::new(dotenv::var("TOKEN").unwrap().as_str(), Handler)?;
    client
        .start_autosharded()
        .context("Failed to start client")?;

    Ok(())
}
