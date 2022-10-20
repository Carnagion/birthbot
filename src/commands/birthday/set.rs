//! Generates and handles the `birthday set` sub-command.

use chrono::DateTime;
use chrono::FixedOffset;
use chrono::NaiveDate;

use mongodb::bson;
use mongodb::bson::Document;

use serenity::builder::CreateApplicationCommandOption;
use serenity::model::application::command::CommandOptionType;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::application::interaction::application_command::CommandDataOption;
use serenity::prelude::Context;
use serenity::utils::Colour;

use crate::errors::BotError;

/// Generates the `birthday set` sub-command.
pub fn create_birthday_set_subcommand(subcommand: &mut CreateApplicationCommandOption) -> &mut CreateApplicationCommandOption {
    subcommand
        .kind(CommandOptionType::SubCommand)
        .name("set")
        .description("Add or update a user's birthday.")
        .create_sub_option(|option| option
            .kind(CommandOptionType::Integer)
            .name("day")
            .description("Day of birth")
            .required(true))
        .create_sub_option(|option| option
            .kind(CommandOptionType::Integer)
            .name("month")
            .description("Month of birth")
            .required(true))
        .create_sub_option(|option| option
            .kind(CommandOptionType::Integer)
            .name("year")
            .description("Year of birth")
            .required(true))
        .create_sub_option(|option| option
            .kind(CommandOptionType::Integer)
            .name("offset")
            .description("Offset from UTC in minutes")
            .required(true))
        .create_sub_option(|option| option
            .kind(CommandOptionType::User)
            .name("user")
            .description("Whose birthday to set")
            .required(false))
}

/// Handles the `birthday set` sub-command.
///
/// # Errors
/// A [BotError] is returned in situations including but not limited to:
/// - One of the required sub-command options is not present or resolved
/// - One of the sub-command options has an invalid value
/// - There was an error connecting to or updating the database
/// - There was an error responding to the command
pub async fn handle_birthday_set_subcommand(subcommand: &CommandDataOption, command: &ApplicationCommandInteraction, context: &Context) -> Result<(), BotError> {
    // Retrieve command options
    let day = *require_command_int_option!(subcommand.options.get(0), "day")? as i32;
    let month = *require_command_int_option!(subcommand.options.get(1), "month")? as i32;
    let year = *require_command_int_option!(subcommand.options.get(2), "year")? as i32;
    let offset = *require_command_int_option!(subcommand.options.get(3), "offset")? as i32;
    let timezone = FixedOffset::east_opt((offset) * 60)
        .ok_or(BotError::UserError(String::from("The offset provided is invalid.")))?;
    let naive = NaiveDate::from_ymd_opt(year, month as u32, day as u32)
        .ok_or(BotError::UserError(String::from("The date provided is invalid.")))?
        .and_hms(0, 0, 0);
    let date = DateTime::<FixedOffset>::from_utc(naive, timezone);
    let guild = command.guild_id
        .ok_or(BotError::UserError(String::from("This command can only be performed in a guild.")))?;
    // Build query and operation documents
    let query = bson_birthday!(command.user.id.0 as i64);
    let operation = bson::doc! {
        "$set": {
            "birth.day": day,
            "birth.month": month,
            "birth.year": year,
            "birth.offset": offset,
        },
    };
    // Connect to database and find collection
    let database = super::connect_mongodb().await?;
    let collection = database.collection::<Document>(guild.to_string().as_str());
    // Insert or replace document
    let replacement = collection
        .find_one_and_update(query, operation, None)
        .await?;
    match replacement {
        None => {
            let insertion = bson::doc! {
                "user": command.user.id.0 as i64,
                "birth": {
                    "day": day,
                    "month": month,
                    "year": year,
                    "offset": offset,
                },
            };
            collection
                .insert_one(insertion, None)
                .await?;
            respond_birthday_set(&date, "added", command, context).await
        },
        Some(_) => respond_birthday_set(&date, "updated", command, context).await,
    }
}

async fn respond_birthday_set(date: &DateTime<FixedOffset>, action: impl Into<String>, command: &ApplicationCommandInteraction, context: &Context) -> Result<(), BotError> {
    command_response!(command, context, |data| data
        .ephemeral(true)
        .embed(|embed| embed
            .title("Success")
            .description(format!("Your birthday was successfully {}.", action.into()))
            .field("Birthday", date.date(), true)
            .colour(Colour::from_rgb(87, 242, 135))))
}