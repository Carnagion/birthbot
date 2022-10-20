//! Generates and handles the `birthday unset` sub-command.

use mongodb::bson::Document;

use serenity::builder::CreateApplicationCommandOption;
use serenity::model::application::command::CommandOptionType;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::prelude::Context;
use serenity::utils::Colour;

use crate::errors::BotError;

/// Generates the `birthday unset` sub-command.
pub fn create_birthday_unset_subcommand(subcommand: &mut CreateApplicationCommandOption) -> &mut CreateApplicationCommandOption {
    subcommand
        .kind(CommandOptionType::SubCommand)
        .name("unset")
        .description("Remove a user's birthday.")
}

/// Handles the `birthday unset` sub-command.
///
/// # Errors
/// A [BotError] is returned in situations including but not limited to:
/// - The required sub-command option is not present or resolved
/// - The sub-command option has an invalid value
/// - There was an error connecting to or updating the database
/// - There was an error responding to the command
pub async fn handle_birthday_unset_subcommand(command: &ApplicationCommandInteraction, context: &Context) -> Result<(), BotError> {
    let guild = command.guild_id
        .ok_or(BotError::UserError(String::from("This command can only be performed in a guild.")))?;
    // Build query document
    let query = bson_birthday!(command.user.id.0 as i64);
    // Connect to database and find collection
    let database = super::connect_mongodb().await?;
    let collection = database.collection::<Document>(guild.to_string().as_str());
    // Delete document
    let result = collection
        .find_one_and_delete(query, None)
        .await?;
    respond_birthday_unset(result, command, context).await
}

async fn respond_birthday_unset(result: Option<Document>, command: &ApplicationCommandInteraction, context: &Context) -> Result<(), BotError> {
    match result {
        None => {
            // If query returned nothing, birthday has not been set yet
            command_response!(command, context, |data| data
                .ephemeral(true)
                .embed(|embed| embed
                    .title("Error")
                    .description("You haven't set a birthday yet.")
                    .colour(Colour::from_rgb(237, 66, 69))))
        },
        Some(_) => {
            // If query returned a document, birthday was removed
            command_response!(command, context, |data| data
                .ephemeral(true)
                .embed(|embed| embed
                    .title("Success")
                    .description("Your birthday was successfully removed.")
                    .colour(Colour::from_rgb(87, 242, 135))))
        },
    }
}