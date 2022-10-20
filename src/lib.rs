//! A [Discord](https://discord.com) bot API for keeping track of users' birthdays.

#![warn(missing_docs)]

use serenity;
use serenity::client::Context;
use serenity::client::EventHandler;
use serenity::model::application::interaction::Interaction;
use serenity::model::gateway::Ready;

#[macro_use]
mod macros;

pub mod commands;
use commands::birthday;
use commands::birthday::check;

pub mod errors;
use errors::BotError;

/// An [EventHandler] attached to the bot client.
pub struct BotEventHandler;

#[serenity::async_trait]
impl EventHandler for BotEventHandler {
    async fn ready(&self, context: Context, _: Ready) {
        #[cfg(not(feature = "guild"))]
        // Set commands globally
        if let Err(error) = commands::set_global_commands(&context).await {
            eprintln!("{:?}", error);
        }

        #[cfg(feature = "guild")]
        // Set commands for a specific guild
        if let Err(error) = commands::set_guild_commands(&context).await {
            eprintln!("{:?}", error);
        }

        check::create_birthday_scheduler(&context);
    }

    async fn interaction_create(&self, context: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            // Handle commands
            let result = match command.data.name.as_str() {
                "birthday" => birthday::handle_birthday_command(&command, &context).await,
                command_name => Err(BotError::CommandError(format!("The command {} is unrecognised.", command_name))),
            };
            // If the error is a user error, use a specific error message, else use a very general error message
            if let Err(bot_error) = result {
                eprintln!("{:?}", bot_error);
                match bot_error {
                    BotError::UserError(user_error) => command_error!(user_error, &command, &context),
                    _ => command_error!("An unexpected error occurred while processing that command.", &command, &context),
                }
            }
        }
    }
}