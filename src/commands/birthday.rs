use std::env;

use chrono::DateTime;
use chrono::FixedOffset;
use chrono::NaiveDate;

use mongodb::Client;
use mongodb::Database;
use mongodb::bson;
use mongodb::bson::Document;
use mongodb::options::ClientOptions;
use mongodb::options::ResolverConfig;

use serenity::builder::CreateApplicationCommand;
use serenity::builder::CreateApplicationCommandOption;
use serenity::model::application::command::CommandOptionType;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::application::interaction::application_command::CommandDataOption;
use serenity::model::user::User;
use serenity::prelude::Context;
use serenity::utils::Colour;

use crate::errors::BotError;

const CLUSTER_KEY: &str = "CLUSTER";
const DATABASE_KEY: &str = "DATABASE";

/// Generates the `birthday` command and its subcommands.
pub fn create_birthday_command(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("birthday")
        .description("Get or set a user's birthday.")
        .create_option(&create_birthday_get_subcommand)
        .create_option(&create_birthday_set_subcommand)
}

fn create_birthday_get_subcommand(subcommand: &mut CreateApplicationCommandOption) -> &mut CreateApplicationCommandOption{
    subcommand
        .kind(CommandOptionType::SubCommand)
        .name("get")
        .description("Gets a birthday.")
        .create_sub_option(|option| option
            .kind(CommandOptionType::User)
            .name("user")
            .description("Whose birthday to get")
            .required(false))
}

fn create_birthday_set_subcommand(subcommand: &mut CreateApplicationCommandOption) -> &mut CreateApplicationCommandOption {
    subcommand
        .kind(CommandOptionType::SubCommand)
        .name("set")
        .description("Sets a birthday.")
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

/// Handles the `birthday` command and its subcommands.
///
/// # Errors
/// A [BotError] is returned if there is an error including but not limited to:
/// - Accessing the database
/// - Loading environment variables
/// - Resolving command options
///
/// etc.
pub async fn handle_birthday_command(command: &ApplicationCommandInteraction, context: &Context) -> Result<(), BotError> {
    // Retrieve sub-command
    let subcommand = command
        .data
        .options
        .get(0)
        .ok_or(BotError::CommandError(String::from("A sub-command is expected.")))?;
    // Handle sub-command based on name
    match subcommand.name.as_str() {
        "get" => handle_birthday_get_subcommand(subcommand, command, context).await,
        "set" => handle_birthday_set_subcommand(subcommand, command, context).await,
        subcommand_name => Err(BotError::CommandError(format!("The sub-command {} is not recognised.", subcommand_name))),
    }
}

async fn handle_birthday_get_subcommand(subcommand: &CommandDataOption, command: &ApplicationCommandInteraction, context: &Context) -> Result<(), BotError> {
    // Retrieve command options
    let user = require_command_user_option!(subcommand.options.get(0), "user", &command.user);
    let guild = command.guild_id
        .ok_or(BotError::UserError(String::from("This command can only be performed in a guild.")))?;
    // Build query document
    let query = bson_birthday!(user.id.to_string());
    // Connect to database and find collection
    let database = connect_mongodb().await?;
    let collection = database.collection::<Document>(guild.to_string().as_str());
    // Retrieve document
    let result = collection
        .find_one(query, None)
        .await?;
    respond_birthday_get(result, user, command, context)
        .await
}

async fn handle_birthday_set_subcommand(subcommand: &CommandDataOption, command: &ApplicationCommandInteraction, context: &Context) -> Result<(), BotError> {
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
    let query = bson_birthday!(user.id.to_string());
    let document = bson::doc! {
        user.id.to_string(): {
            "birth": {
                "day": day,
                "month": month,
                "year": year,
                "offset": offset,
            },
        },
    };
    // Connect to database and find collection
    let database = connect_mongodb().await?;
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

async fn connect_mongodb() -> Result<Database, BotError> {
    let cluster = env::var(CLUSTER_KEY)?;
    let options = ClientOptions::parse_with_resolver_config(&cluster, ResolverConfig::cloudflare())
        .await?;
    let client = Client::with_options(options)?;
    let database = env::var(DATABASE_KEY)?;
    Ok(client.database(database.as_str()))
}

async fn respond_birthday_get(result: Option<Document>, user: &User, command: &ApplicationCommandInteraction, context: &Context) -> Result<(), BotError> {
    match result {
        // If query returned nothing, birthday has not been set yet
        None => {
            let description = if user.id == command.user.id {
                String::from("You haven't set a birthday yet.")
            } else {
                format!("<@{}> hasn't set a birthday yet.", user.id)
            };
            command_response!(command, context, |data| data
                .ephemeral(true)
                .embed(|embed| embed
                    .title("Error")
                    .description(description)
                    .colour(Colour::from_rgb(237, 66, 69))))
        },
        // If query returned a document, parse and show the birthday
        Some(document) => {
            let birthday = document
                .get_document(user.id.to_string())?
                .get_document("birth")?;
            let day = birthday.get_i32("day")?;
            let month = birthday.get_i32("month")?;
            let year = birthday.get_i32("year")?;
            let offset = birthday.get_i32("offset")?;
            let timezone = FixedOffset::east_opt(offset * 60)
                .ok_or(BotError::CommandError(String::from("The offset stored is invalid.")))?;
            let naive = NaiveDate::from_ymd_opt(year, month as u32, day as u32)
                .ok_or(BotError::UserError(String::from("The date provided is invalid.")))?
                .and_hms(0, 0, 0);
            let date = DateTime::<FixedOffset>::from_utc(naive, timezone);
            let description = if user.id == command.user.id {
                String::from("Your birthday was successfully retrieved.")
            } else {
                format!("<@{}>'s birthday was successfully retrieved.", user.id)
            };
            command_response!(command, context, |data| data
                .ephemeral(true)
                .embed(|embed| embed
                    .title("Success")
                    .description(description)
                    .field("Birthday", date.date(), true)
                    .colour(Colour::from_rgb(87, 242, 135))))
        },
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