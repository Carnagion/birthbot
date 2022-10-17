#[cfg(feature = "guild")]
use std::env;

use serenity;
use serenity::builder::CreateApplicationCommands;
use serenity::client::Context;
use serenity::client::EventHandler;
use serenity::model::application::interaction::Interaction;
use serenity::model::gateway::Ready;
#[cfg(feature = "guild")]
use serenity::model::id::GuildId;
use serenity::model::prelude::command::Command;

mod birthday;

use crate::errors::BotError;
use crate::macros;

#[cfg(feature = "guild")]
const GUILD_KEY: &str = "GUILD";

pub struct BotEventHandler;

#[serenity::async_trait]
impl EventHandler for BotEventHandler {
    async fn ready(&self, context: Context, _: Ready) {
        #[cfg(not(feature = "guild"))]
        if let Err(error) = set_global_commands(&context).await {
            println!("{:?}", error);
        }

        #[cfg(feature = "guild")]
        if let Err(error) = set_guild_commands(&context).await {
            println!("{:?}", error);
        }
    }

    async fn interaction_create(&self, context: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let result = match command.data.name.as_str() {
                "birthday" => birthday::handle_birthday_command(&command, &context).await,
                command_name => Err(BotError::CommandError(format!("The command {} is unrecognised.", command_name))),
            };
            if let Err(bot_error) = result {
                println!("{:?}", bot_error);
                match bot_error {
                    BotError::UserError(user_error) => macros::command_error!(user_error, &command, &context),
                    _ => macros::command_error!("An unexpected error occurred while processing that command.", &command, &context),
                }
            }
        }
    }
}

fn create_commands(commands: &mut CreateApplicationCommands) -> &mut CreateApplicationCommands {
    commands
        .create_application_command(&birthday::create_birthday_command)
}

#[cfg(not(feature = "guild"))]
async fn set_global_commands(context: &Context) -> Result<Vec<Command>, BotError> {
    Command::set_global_application_commands(&context.http, &create_commands)
        .await
        .map_err(BotError::SerenityError)
}

#[cfg(feature = "guild")]
async fn set_guild_commands(context: &Context) -> Result<Vec<Command>, BotError> {
    Ok(GuildId(env::var(GUILD_KEY)?
            .parse()?)
        .set_application_commands(&context.http, &create_commands)
        .await?)
}