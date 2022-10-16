use std::env;

use dotenv;

use serenity::Client;
use serenity::prelude::GatewayIntents;

mod commands;
mod macros;

use crate::commands::BirthdayCommandHandler;

const TOKEN_KEY: &str = "TOKEN";

#[tokio::main]
async fn main() {
    // Load .env file
    dotenv::dotenv()
        .expect("The .env file could not be loaded.");

    // Retrieve token
    let token = env::var(TOKEN_KEY)
        .expect("The client token is invalid or non-existent.");
    
    // Initialise bot client
    Client::builder(token, GatewayIntents::GUILDS)
        .event_handler(BirthdayCommandHandler)
        .await
        .expect("The command handler could not be attached to the client.")
        .start()
        .await
        .expect("The client could not be started.");
}