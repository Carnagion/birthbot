//! Generates and handles the `birthday list` sub-command.

use mongodb::bson::Document;

use serenity::builder::CreateApplicationCommandOption;
use serenity::model::application::command::CommandOptionType;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::application::interaction::application_command::CommandDataOption;
use serenity::prelude::Context;
use serenity::utils::Colour;

use crate::errors::BotError;

/// Generates the `birthday list` sub-command.
pub fn create_birthday_list_subcommand(subcommand: &mut CreateApplicationCommandOption) -> &mut CreateApplicationCommandOption {
    subcommand
        .kind(CommandOptionType::SubCommand)
        .name("list")
        .description("Retrieve all birthdays.")
        .create_sub_option(|option| option
            .kind(CommandOptionType::Boolean)
            .name("sorted")
            .description("Sort displayed birthdays")
            .required(false))
}

/// Handles the `birthday list` sub-command.
///
/// # Errors
/// A [BotError] is returned in situations including but not limited to:
/// - The sub-command option is not resolved or has an invalid value
/// - There was an error connecting to or querying the database
/// - There was an error responding to the command
pub async fn handle_birthday_list_subcommand(subcommand: &CommandDataOption, command: &ApplicationCommandInteraction, context: &Context) -> Result<(), BotError> {
    // Retrieve command options
    let sorted = *require_command_simple_option!(subcommand.options.get(0), Boolean, "sorted", true)?;
    let guild = command.guild_id
        .ok_or(BotError::UserError(String::from("This command can only be performed in a guild.")))?;
    // Build query document
    let query = bson_birthday!();
    // Connect to database and retrieve all documents
    let mut cursor = super::connect_mongodb()
        .await?
        .collection::<Document>(guild.to_string().as_str())
        .find(query, None)
        .await?;
    let mut birthdays = vec![];
    while cursor.advance().await? {
        let document = cursor.deserialize_current()?;
        let user = document.get_i64("user")?;
        let birth = super::get_birthday(&document)?;
        birthdays.push((user, birth));
    }
    if sorted {
        birthdays.sort_by(|(_, left), (_, right)| left.cmp(right));
    }
    command_response!(command, context, |data| data
        .ephemeral(true)
        .embed(|embed| {
            embed
                .title("Success")
                .description("All birthdays were successfully retrieved.")
                .colour(Colour::from_rgb(87, 242, 135));
            for (user, birth) in birthdays {
                embed.field("Birthday", format!("<@{}> ({})", user, birth.date()), true);
            }
            embed
        }))
}