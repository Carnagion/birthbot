use std::env;

use mongodb::Client;
use mongodb::Database;
use mongodb::bson;
use mongodb::bson::Document;
use mongodb::bson::DateTime;
use mongodb::options::ClientOptions;
use mongodb::options::ResolverConfig;

use serenity::builder::CreateApplicationCommand;
use serenity::builder::CreateApplicationCommandOption;
use serenity::model::application::command::CommandOptionType;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::application::interaction::application_command::CommandDataOption;
use serenity::model::application::interaction::application_command::CommandDataOptionValue;
use serenity::model::id::GuildId;
use serenity::prelude::Context;

use crate::macros;

const CLUSTER_URI_KEY: &str = "CLUSTER";
const DATABASE_KEY: &str = "DATABASE";

pub fn create_birthday_command(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("birthday")
        .description("Birthday-related commands.")
        .create_option(&create_birthday_set_subcommand)
        .create_option(&create_birthday_get_subcommand)
}

fn create_birthday_get_subcommand(subcommand: &mut CreateApplicationCommandOption) -> &mut CreateApplicationCommandOption {
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

pub async fn handle_birthday_command(command: &ApplicationCommandInteraction, context: &Context) {
    match command.guild_id {
        None => macros::command_response!("Error: this command can only be executed in a guild.", command, context),
        Some(guild_id) => {
            match command.data.options.get(0) {
                None => macros::command_response!("Error: a sub-command is expected.", command, context),
                Some(subcommand) => {
                    match subcommand.name.as_str() {
                        "get" => handle_birthday_get_subcommand(subcommand, command, context, &guild_id).await,
                        "set" => handle_birthday_set_subcommand(subcommand, command, context, &guild_id).await,
                        subcommand_name => macros::command_response!(format!(r#"Error: the sub-command "{}" is not recognised."#, subcommand_name), command, context),
                    }
                },
            }
        },
    }
}

async fn handle_birthday_get_subcommand(subcommand: &CommandDataOption, command: &ApplicationCommandInteraction, context: &Context, guild_id: &GuildId) {
    if let Some(user) = macros::require_command_user_or_default!(subcommand.options.get(0), command, context) {
        match connect_mongodb().await {
            Err(error) => macros::command_response!(error, command, context),
            Ok(database) => {
                let filter = bson::doc! {
                    "user": user.id.to_string(),
                };
                let result = database
                    .collection::<Document>(guild_id.to_string().as_str())
                    .find_one(filter, None)
                    .await;
                match result {
                    Err(error) => {
                        macros::command_response!("Error: something went wrong unexpectedly.", command, context);
                        println!("Error: {:?}", error);
                    },
                    Ok(option) => {
                        match option {
                            None => {
                                let message = if user.id == command.user.id {
                                    String::from("You haven't set a birthday yet.")
                                } else {
                                    format!("<@{}> hasn't set a birthday yet.", user.id)
                                };
                                macros::command_response!(message, command, context);
                            },
                            Some(document) => {
                                match document.get_datetime("birthday") {
                                    Err(error) => macros::command_response!(format!("Error: {}.", error), command, context),
                                    Ok(date) => {
                                        let message = if user.id == command.user.id {
                                            format!("Your birthday is on {}.", date)
                                        } else {
                                            format!("<@{}>'s birthday is on {}.", user.id, date)
                                        };
                                        macros::command_response!(message, command, context);
                                    },
                                }
                            },
                        }
                    }
                }
            }
        }
    }
}

async fn handle_birthday_set_subcommand(subcommand: &CommandDataOption, command: &ApplicationCommandInteraction, context: &Context, guild_id: &GuildId) {
    let option_day = macros::require_command_option!(subcommand.options.get(0), "day", Integer, command, context);
    let option_month = macros::require_command_option!(subcommand.options.get(1), "month", Integer, command, context);
    let option_year = macros::require_command_option!(subcommand.options.get(2), "year", Integer, command, context);
    if let (Some(day), Some(month), Some(year)) = (option_day, option_month, option_year) {
        let builder = DateTime::builder()
            .year(*year as i32)
            .month(*month as u8)
            .day(*day as u8)
            .build();
        match builder {
            Err(error) => macros::command_response!(format!("Error: {}.", error), command, context),
            Ok(date) => {
                if let Some(user) = macros::require_command_user_or_default!(subcommand.options.get(3), command, context) {
                    match connect_mongodb().await {
                        Err(error) => macros::command_response!(error, command, context),
                        Ok(database) => {
                            let filter = bson::doc! {
                                "user": user.id.to_string(),
                            };
                            let replacement = bson::doc! {
                                "user": user.id.to_string(),
                                "birthday": date,
                            };
                            let collection = database.collection::<Document>(guild_id.to_string().as_str());
                            let replace_result = collection
                                .find_one_and_replace(filter, &replacement, None)
                                .await;
                            match replace_result {
                                Err(error) => {
                                    macros::command_response!("Error: something went wrong unexpectedly.", command, context);
                                    println!("Error: {:?}", error);
                                },
                                Ok(Some(_)) => {
                                    let message = if user.id == command.user.id {
                                        String::from("Your birthday was successfully updated.")
                                    } else {
                                        format!("<@{}>'s birthday was successfully updated.", user.id)
                                    };
                                    macros::command_response!(message, command, context);
                                },
                                Ok(None) => {
                                    let insert_result = collection
                                        .insert_one(&replacement, None)
                                        .await;
                                    match insert_result {
                                        Err(error) => {
                                            macros::command_response!("Error: something went wrong unexpectedly.", command, context);
                                            println!("Error: {:?}", error);
                                        },
                                        Ok(_) => {
                                            let message = if user.id == command.user.id {
                                                String::from("Your birthday was successfully set.")
                                            } else {
                                                format!("<@{}>'s birthday was successfully set.", user.id)
                                            };
                                            macros::command_response!(message, command, context);
                                        },
                                    }
                                },
                            }
                        }
                    }
                }
            }
        }
    }
}

async fn connect_mongodb() -> Result<Database, String> {
    let uri = env::var(CLUSTER_URI_KEY)
        .map_err(|error| format!("Error: {}.", error))?;
    let client_options = ClientOptions::parse_with_resolver_config(&uri, ResolverConfig::cloudflare())
        .await
        .map_err(|error| {
            println!("Error: {:?}", error);
            String::from("Error: something went wrong unexpectedly.")
        })?;
    let client = Client::with_options(client_options)
        .map_err(|error| {
            println!("Error: {:?}", error);
            String::from("Error: something went wrong unexpectedly.")
        })?;
    let database = env::var(DATABASE_KEY)
        .map_err(|error| format!("Error: {}.", error))?;
    Ok(client.database(database.as_str()))
}