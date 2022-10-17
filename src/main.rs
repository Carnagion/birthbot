use std::env;

use dotenv;

use serenity::Client;
use serenity::prelude::GatewayIntents;

use tokio;

mod commands;
use commands::BotEventHandler;

mod errors;
use errors::BotError;

mod macros;

const TOKEN_KEY: &str = "TOKEN";

#[tokio::main]
async fn main() -> Result<(), BotError> {
    dotenv::dotenv()?;
    Client::builder(env::var(TOKEN_KEY)?, GatewayIntents::empty())
        .event_handler(BotEventHandler)
        .await?
        .start()
        .await?;
    Ok(())
}