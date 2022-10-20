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
use serenity::model::user::User;
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
    let user = require_command_user_option!(subcommand.options.get(4), "user", &command.user);
    let guild = command.guild_id
        .ok_or(BotError::UserError(String::from("This command can only be performed in a guild.")))?;
    // Build query and replacement documents
    let query = bson_birthday!(user.id.0 as i64);
    let document = bson::doc! {
        "user": user.id.0 as i64,
        "birth": {
            "day": day,
            "month": month,
            "year": year,
            "offset": offset,
        },
    };
    // Connect to database and find collection
    let database = super::connect_mongodb().await?;
    let collection = database.collection::<Document>(guild.to_string().as_str());
    // Insert or replace document
    let replacement = collection
        .find_one_and_replace(query, &document, None)
        .await?;
    match replacement {
        None => {
            collection
                .insert_one(&document, None)
                .await?;
            respond_birthday_set(&date, "set", user, command, context).await
        },
        Some(_) => respond_birthday_set(&date, "updated", user, command, context).await,
    }
}

async fn respond_birthday_set(date: &DateTime<FixedOffset>, action: impl Into<String>, user: &User, command: &ApplicationCommandInteraction, context: &Context) -> Result<(), BotError> {
    let description = if user.id == command.user.id {
        format!("Your birthday was successfully {}.", action.into())
    } else {
        format!("<@{}>'s birthday was successfully {}.", user.id, action.into())
    };
    command_response!(command, context, |data| data
        .ephemeral(true)
        .embed(|embed| embed
            .title("Success")
            .description(description)
            .field("Birthday", date.date(), true)
            .colour(Colour::from_rgb(87, 242, 135))))
}