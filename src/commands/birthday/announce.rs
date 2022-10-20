//! Generates and handles the `birthday announce` sub-command.

use mongodb::bson;
use mongodb::bson::Document;

use serenity::builder::CreateApplicationCommandOption;
use serenity::model::application::command::CommandOptionType;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::application::interaction::application_command::CommandDataOption;
use serenity::model::channel::PartialChannel;
use serenity::prelude::Context;
use serenity::utils::Colour;

use crate::errors::BotError;

/// Generates the `birthday announce` sub-command.
pub fn create_birthday_announce_subcommand(subcommand: &mut CreateApplicationCommandOption) -> &mut CreateApplicationCommandOption {
    subcommand
        .kind(CommandOptionType::SubCommand)
        .name("announce")
        .description("Add or update the channel for announcing birthdays.")
        .create_sub_option(|option| option
            .kind(CommandOptionType::Channel)
            .name("channel")
            .description("The channel for birthday announcements")
            .required(true))
}

/// Handles the `birthday announce` sub-command.
///
/// # Errors
/// A [BotError] is returned in situations including but not limited to:
/// - The sub-command option is not resolved or has an invalid value
/// - There was an error connecting to, querying, or updating the database
/// - There was an error responding to the command
pub async fn handle_birthday_announce_subcommand(subcommand: &CommandDataOption, command: &ApplicationCommandInteraction, context: &Context) -> Result<(), BotError> {
    // Retrieve command options
    let channel = require_command_channel_option!(subcommand.options.get(0), "channel")?;
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
        "$set": {
            "config.channel": channel.id.0 as i64,
        },
    };
    // Connect to database and find collection
    let database = super::connect_mongodb().await?;
    let collection = database.collection::<Document>(guild.to_string().as_str());
    // Update or insert document
    let result = collection
        .find_one_and_update(query, &operation, None)
        .await?;
    match result {
        None => {
            collection
                .insert_one(&operation, None)
                .await?;
            respond_birthday_announce(channel, "added", command, context).await
        },
        Some(_) => respond_birthday_announce(channel, "updated", command, context).await,
    }
}

async fn respond_birthday_announce(channel: &PartialChannel, action: impl Into<String>, command: &ApplicationCommandInteraction, context: &Context) -> Result<(), BotError> {
    command_response!(command, context, |data| data
        .embed(|embed| embed
            .title("Success")
            .description(format!("The birthday announcement channel was successfully {}.", action.into()))
            .field("Channel", format!("<#{}>", channel.id), true)
            .colour(Colour::from_rgb(87, 242, 135))))
}