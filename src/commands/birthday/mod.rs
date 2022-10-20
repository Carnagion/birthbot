//! Generates and handles the `birthday` command and its sub-commands.

use std::env;

use chrono::DateTime;
use chrono::FixedOffset;
use chrono::NaiveDate;

use mongodb::Client;
use mongodb::Database;
use mongodb::bson::Document;
use mongodb::options::ClientOptions;
use mongodb::options::ResolverConfig;

use serenity::builder::CreateApplicationCommand;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::prelude::Context;

pub mod announce;
pub mod check;
pub mod get;
pub mod set;
pub mod unannounce;
pub mod unset;

use crate::errors::BotError;

const CLUSTER_KEY: &str = "CLUSTER";
const DATABASE_KEY: &str = "DATABASE";

/// Generates the `birthday` command and its subcommands.
pub fn create_birthday_command(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("birthday")
        .description("Get or set a user's birthday.")
        .create_option(announce::create_birthday_announce_subcommand)
        .create_option(unannounce::create_birthday_unannounce_subcommand)
        .create_option(get::create_birthday_get_subcommand)
        .create_option(set::create_birthday_set_subcommand)
        .create_option(unset::create_birthday_unset_subcommand)
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
        "announce" => announce::handle_birthday_announce_subcommand(subcommand, command, context).await,
        "get" => get::handle_birthday_get_subcommand(subcommand, command, context).await,
        "set" => set::handle_birthday_set_subcommand(subcommand, command, context).await,
        "unannounce" => unannounce::handle_birthday_announce_subcommand(command, context).await,
        "unset" => unset::handle_birthday_unset_subcommand(subcommand, command, context).await,
        subcommand_name => Err(BotError::CommandError(format!("The sub-command {} is not recognised.", subcommand_name))),
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

fn get_birthday(document: &Document) -> Result<DateTime<FixedOffset>, BotError> {
    let birth = document.get_document("birth")?;
    let day = birth.get_i32("day")?;
    let month = birth.get_i32("month")?;
    let year = birth.get_i32("year")?;
    let offset = birth.get_i32("offset")?;
    let timezone = FixedOffset::east_opt(offset * 60)
        .ok_or(BotError::CommandError(String::from("The offset stored is invalid.")))?;
    let naive = NaiveDate::from_ymd_opt(year, month as u32, day as u32)
        .ok_or(BotError::UserError(String::from("The date provided is invalid.")))?
        .and_hms(0, 0, 0);
    Ok(DateTime::<FixedOffset>::from_utc(naive, timezone))
}