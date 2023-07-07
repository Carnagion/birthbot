//! A birthday bot for Discord that is ad-free, easy to use, and stores minimal data.

#![warn(missing_docs)]

use std::{fs, sync::Arc, time::Duration};

use mongodm::{mongo::options::ResolverConfig, prelude::*};

use poise::{builtins, serenity_prelude as serenity, FrameworkOptions};

use serenity::*;

use snafu::ResultExt;

pub mod prelude;
use prelude::*;

pub mod config;

pub mod model;
pub use model::Birthday;

pub mod commands;

pub mod tasks;

mod bot_data;
pub use bot_data::*;

mod error;
pub use error::BotError;

/// An abstraction over a [`BotFramework`], allowing for easy instantiation with all the relevant commands, tasks, handlers, and configuration.
#[derive(Clone)]
pub struct Birthbot(Arc<BotFramework>);

impl Birthbot {
    /// Creates a new instance of [`Birthbot`] with the provided configuration.
    ///
    /// # Errors
    ///
    /// Fails if building the inner [`BotFramework`] fails.
    /// See [`BotFramework::builder`].
    pub async fn new(config: BirthbotConfig) -> BotResult<Self> {
        let framework = BotFramework::builder()
            .token(&config.bot_token)
            .intents(GatewayIntents::non_privileged())
            .options(FrameworkOptions {
                commands: vec![commands::birthday()],
                on_error: |error| Box::pin(util::report_framework_error(error)),
                ..Default::default()
            })
            .setup(move |context, _, framework| {
                Box::pin(setup_bot_data(context, framework, config))
            })
            .build()
            .await?;
        Ok(Self(framework))
    }

    /// Starts the bot and keeps it running in an asynchronous loop.
    pub async fn start(self) -> BotResult<()> {
        self.0.start().await?;
        Ok(())
    }
}

async fn setup_bot_data(
    context: &Context,
    framework: &BotFramework,
    config: BirthbotConfig,
) -> BotResult<BotData> {
    // Setup the bot data and database
    let data = BotData {
        database: setup_database(&config.database.cluster_uri, &config.database.name).await?,
        birthday_check_interval: Duration::from_secs(config.birthday_check_interval.unwrap_or(900)),
    };

    // Register the commands either globally or in the test guild
    let commands = builtins::create_application_commands(&framework.options().commands);
    match config.test_guild_id {
        None => {
            Command::set_global_application_commands(&context.http, |global_commands| {
                *global_commands = commands;
                global_commands
            })
            .await
        },
        Some(guild_id) => {
            guild_id
                .set_application_commands(&context.http, |app_commands| {
                    *app_commands = commands;
                    app_commands
                })
                .await
        },
    }?;

    // Start the birthday announcement checker
    tasks::schedule_birthday_announcer(context.clone(), data.clone())?;

    // Start the update poster
    let updates = match config.updates_path {
        None => None,
        Some(updates_path) => Some(
            fs::read_to_string(&updates_path).with_context(|_| FileSnafu { path: updates_path })?,
        ),
    };
    if let Some(updates) = updates {
        tasks::announce_updates(context, &data, &updates).await?;
    }

    Ok(data)
}

async fn setup_database(uri: &str, database: &str) -> BotResult<MongoDatabase> {
    // Connect to the database
    let client_options =
        MongoClientOptions::parse_with_resolver_config(uri, ResolverConfig::cloudflare()).await?;
    let client = MongoClient::with_options(client_options)?;
    Ok(client.database(database))
}
