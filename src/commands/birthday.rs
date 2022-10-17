use std::env;

use mongodb::Client;
use mongodb::Database;
use mongodb::bson;
use mongodb::bson::DateTime;
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

use crate::errors::BotError;
use crate::macros;

const CLUSTER_KEY: &str = "CLUSTER";
const DATABASE_KEY: &str = "DATABASE";

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
        .description("Get a user's birthday.")
        .create_sub_option(|option| option
            .kind(CommandOptionType::User)
            .name("user")
            .description("The user whose birthday to get.")
            .required(false))
}

fn create_birthday_set_subcommand(subcommand: &mut CreateApplicationCommandOption) -> &mut CreateApplicationCommandOption {
    subcommand
        .kind(CommandOptionType::SubCommand)
        .name("set")
        .description("Set a user's birthday.")
        .create_sub_option(|option| option
            .kind(CommandOptionType::Integer)
            .name("day")
            .description("The day of birth.")
            .required(true))
        .create_sub_option(|option| option
            .kind(CommandOptionType::Integer)
            .name("month")
            .description("The month of birth.")
            .required(true))
        .create_sub_option(|option| option
            .kind(CommandOptionType::Integer)
            .name("year")
            .description("The year of birth.")
            .required(true))
        .create_sub_option(|option| option
            .kind(CommandOptionType::User)
            .name("user")
            .description("The user whose birthday to set.")
            .required(false))
}

pub async fn handle_birthday_command(command: &ApplicationCommandInteraction, context: &Context) -> Result<(), BotError> {
    let subcommand = command
        .data
        .options
        .get(0)
        .ok_or(BotError::CommandError(String::from("A sub-command is expected.")))?;
    match subcommand.name.as_str() {
        "get" => handle_birthday_get_subcommand(subcommand, command, context).await,
        "set" => handle_birthday_set_subcommand(subcommand, command, context).await,
        subcommand_name => Err(BotError::CommandError(format!("The sub-command {} is not recognised.", subcommand_name))),
    }
}

async fn handle_birthday_get_subcommand(subcommand: &CommandDataOption, command: &ApplicationCommandInteraction, context: &Context) -> Result<(), BotError> {
    let user = macros::require_command_user_option!(subcommand.options.get(0), "user", &command.user);
    let guild = command.guild_id
        .ok_or(BotError::CommandError(String::from("This command can only be performed in a guild.")))?;
    let query = bson::doc! {
        user.id.to_string().as_str(): {
            "$exists": true,
            "$type": "date",
        },
    };
    let database = connect_mongodb().await?;
    let collection = database.collection::<Document>(guild.to_string().as_str());
    let result = collection
        .find_one(query, None)
        .await?;
    let message = birthday_get_message(result, user, command)?;
    macros::command_response!(message, command, context, true)
        .map_err(BotError::SerenityError)
}

async fn handle_birthday_set_subcommand(subcommand: &CommandDataOption, command: &ApplicationCommandInteraction, context: &Context) -> Result<(), BotError> {
    let day = macros::require_command_int_option!(subcommand.options.get(0), "day")?;
    let month = macros::require_command_int_option!(subcommand.options.get(1), "month")?;
    let year = macros::require_command_int_option!(subcommand.options.get(2), "year")?;
    let date = DateTime::builder()
        .year(*year as i32)
        .month(*month as u8)
        .day(*day as u8)
        .build()
        .map_err(|_| BotError::CommandError(String::from("The date provided is invalid.")))?;
    let user = macros::require_command_user_option!(subcommand.options.get(3), "user", &command.user);
    let guild = command.guild_id
        .ok_or(BotError::CommandError(String::from("This command can only be performed in a guild.")))?;
    let query = bson::doc! {
        user.id.to_string(): {
            "$exists": true,
            "$type": "date",
        },
    };
    let document = bson::doc! {
        user.id.to_string(): date,
    };
    let database = connect_mongodb().await?;
    let collection = database.collection::<Document>(guild.to_string().as_str());
    let replacement = collection
        .find_one_and_replace(query, &document, None)
        .await?;
    let message = match replacement {
        None => {
            collection
                .insert_one(&document, None)
                .await?;
            birthday_set_message("set", user, command)
        },
        Some(_) => birthday_set_message("updated", user, command),
    };
    macros::command_response!(message, command, context, true)
        .map_err(BotError::SerenityError)
}

async fn connect_mongodb() -> Result<Database, BotError> {
    let cluster = env::var(CLUSTER_KEY)?;
    let options = ClientOptions::parse_with_resolver_config(&cluster, ResolverConfig::cloudflare())
        .await?;
    let client = Client::with_options(options)?;
    let database = env::var(DATABASE_KEY)?;
    Ok(client.database(database.as_str()))
}

fn birthday_get_message(result: Option<Document>, user: &User, command: &ApplicationCommandInteraction) -> Result<String, BotError> {
    match result {
        None => {
            Ok(if user.id == command.user.id {
                String::from("You haven't set a birthday yet.")
            } else {
                format!("<@{}> hasn't set a birthday yet.", user.id)
            })
        },
        Some(document) => {
            let date = document.get_datetime(user.id.to_string())?;
            Ok(if user.id == command.user.id {
                format!("Your birthday is on {}.", date)
            } else {
                format!("<@{}>'s birthday is on {}.", user.id, date)
            })
        },
    }
}

fn birthday_set_message(action: impl Into<String>, user: &User, command: &ApplicationCommandInteraction) -> String {
    if user.id == command.user.id {
        format!("Your birthday was successfully {}.", action.into())
    } else {
        format!("<@{}>'s birthday was successfully {}.", user.id, action.into())
    }
}