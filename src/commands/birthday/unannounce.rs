//! Generates and handles the `birthday unannounce` sub-command.

use mongodb::bson;
use mongodb::bson::Document;

use serenity::builder::CreateApplicationCommandOption;
use serenity::model::application::command::CommandOptionType;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::prelude::Context;
use serenity::utils::Colour;

use crate::errors::BotError;

/// Generates the `birthday unannounce` sub-command.
pub fn create_birthday_unannounce_subcommand(subcommand: &mut CreateApplicationCommandOption) -> &mut CreateApplicationCommandOption {
    subcommand
        .kind(CommandOptionType::SubCommand)
        .name("unannounce")
        .description("Remove the channel for announcing birthdays.")
}

/// Handles the `birthday unannounce` sub-command.
///
/// # Errors
/// A [BotError] is returned in situations including but not limited to:
/// - There was an error connecting to, querying, or updating the database
/// - There was an error responding to the command
pub async fn handle_birthday_unannounce_subcommand(command: &ApplicationCommandInteraction, context: &Context) -> Result<(), BotError> {
    let guild = command.guild_id
        .ok_or(BotError::UserError(String::from("This command can only be performed in a guild.")))?;
    // Build query and operation documents
    let query = bson::doc! {
        "config": {
            "$exists": true,
            "$type": "object",
        },
    };
    let operation = bson::doc! {
        "$unset": {
            "config.channel": "",
        },
    };
    // Connect to database and find collection
    let database = super::connect_mongodb().await?;
    let collection = database.collection::<Document>(guild.to_string().as_str());
    // Update document
    collection
        .find_one_and_update(query, operation, None)
        .await?;
    command_response!(command, context, |data| data
        .embed(|embed| embed
            .title("Success")
            .description("The birthday announcement channel was successfully removed.")
            .colour(Colour::from_rgb(87, 242, 135))))
}