#[cfg(feature = "guild")]
use std::env;

use serenity::builder::CreateApplicationCommands;
use serenity::client::Context;
use serenity::client::EventHandler;
use serenity::model::application::interaction::Interaction;
use serenity::model::gateway::Ready;
#[cfg(feature = "guild")]
use serenity::model::id::GuildId;
use serenity::model::prelude::command::Command;

mod birthday;

use crate::macros;

#[cfg(feature = "guild")]
const GUILD_ID_KEY: &str = "GUILD";

pub struct BirthdayCommandHandler;

#[serenity::async_trait]
impl EventHandler for BirthdayCommandHandler {
    async fn ready(&self, context: Context, _ready: Ready) {
        #[cfg(not(feature = "guild"))]
        // Set global commands
        if let Err(error) = Command::set_global_application_commands(&context.http, &create_application_commands).await {
            println!("{}", error);
        }

        #[cfg(feature = "guild")]
        // Set commands specific to guild
        GuildId(env::var(GUILD_ID_KEY)
                .expect("todo")
                .parse()
                .expect("todo"))
            .set_application_commands(&context.http, &create_application_commands)
            .await
            .expect("todo");
    }

    async fn interaction_create(&self, context: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            match command.data.name.as_str() {
                "birthday" => birthday::handle_birthday_command(&command, &context).await,
                command_name => macros::command_response!(format!(r#"Unrecognised command "{command_name}"#), &command, &context),
            }
        }
    }
}

fn create_application_commands(commands: &mut CreateApplicationCommands) -> &mut CreateApplicationCommands {
    commands
        .create_application_command(&birthday::create_birthday_command)
}