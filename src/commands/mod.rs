//! Contains functions to generate and handle bot slash commands.

#[cfg(feature = "guild")]
use std::env;

use serenity;
use serenity::builder::CreateApplicationCommands;
use serenity::client::Context;
#[cfg(feature = "guild")]
use serenity::model::id::GuildId;
use serenity::model::prelude::command::Command;

pub mod birthday;
use birthday::check;

use crate::errors::BotError;

#[cfg(feature = "guild")]
const GUILD_KEY: &str = "GUILD";

#[cfg(not(feature = "guild"))]
/// Overwrites global slash commands with the generated slash commands.
///
/// # Errors
/// A [BotError] is returned if there are any Serenity API errors while setting the commands.
pub async fn set_global_commands(context: &Context) -> Result<Vec<Command>, BotError> {
    Command::set_global_application_commands(&context.http, &create_commands)
        .await
        .map_err(BotError::SerenityError)
}

#[cfg(feature = "guild")]
/// Overwrites a guild's slash commands with the generated slash commands.
/// The guild ID is retrieved from a `.env` file at runtime.
///
/// # Errors
/// A [BotError] is returned if there are any Serenity API errors while setting the commands.
pub async fn set_guild_commands(context: &Context) -> Result<Vec<Command>, BotError> {
    Ok(GuildId(env::var(GUILD_KEY)?
            .parse()?)
        .set_application_commands(&context.http, &create_commands)
        .await?)
}

fn create_commands(commands: &mut CreateApplicationCommands) -> &mut CreateApplicationCommands {
    check::setup_birthday_cron();
    commands
        .create_application_command(birthday::create_birthday_command)
}