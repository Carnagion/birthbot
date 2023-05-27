use std::{num::ParseIntError, time::Duration};

use dotenvy::Error as DotEnvError;

use envy::Error as EnvError;

use mongodm::{mongo::options::ResolverConfig, prelude::*};

use poise::{
    builtins,
    serenity_prelude::{Command, Context, Error as DiscordError, GatewayIntents, GuildId},
    FrameworkOptions,
};

use serde::{Deserialize, Serialize};

use snafu::Snafu;

use birthbot::{commands, prelude::*, tasks};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
struct Config {
    birthbot_token: String,
    birthbot_mongodb_uri: String,
    birthbot_mongodb_database: String,
    birthbot_birthday_check_interval: u32,
    birthbot_test_guild_id: Option<GuildId>,
}

#[derive(Debug, Snafu)]
enum StartupError {
    #[snafu(context(false))]
    DotEnv { source: DotEnvError },
    #[snafu(context(false))]
    Env { source: EnvError },
    #[snafu(context(false))]
    NumParse { source: ParseIntError },
    #[snafu(context(false))]
    Discord { source: DiscordError },
    #[snafu(context(false))]
    Mongodb { source: MongoError },
}

#[tokio::main]
async fn main() -> Result<(), StartupError> {
    dotenvy::dotenv()?;

    let config = envy::from_env::<Config>()?;

    BotFramework::builder()
        .token(config.birthbot_token)
        .intents(GatewayIntents::non_privileged())
        .options(FrameworkOptions {
            commands: vec![commands::birthday()],
            on_error: |error| Box::pin(util::report_framework_error(error)),
            ..Default::default()
        })
        .setup(move |context, _, framework| {
            Box::pin(async move {
                let data = BotData {
                    database: connect_mongodb(
                        &config.birthbot_mongodb_uri,
                        &config.birthbot_mongodb_database,
                    )
                    .await?,
                    birthday_check_interval: Duration::from_secs(
                        (config.birthbot_birthday_check_interval * 60) as u64,
                    ),
                };

                register_commands(
                    context,
                    &framework.options().commands,
                    config.birthbot_test_guild_id,
                )
                .await?;

                tasks::schedule_birthday_announcer(context.clone(), data.clone())?;

                Ok(data)
            })
        })
        .build()
        .await?
        .start()
        .await?;

    Ok(())
}

async fn connect_mongodb(uri: &str, database: &str) -> Result<MongoDatabase, MongoError> {
    let client_options =
        MongoClientOptions::parse_with_resolver_config(uri, ResolverConfig::cloudflare()).await?;
    let client = MongoClient::with_options(client_options)?;
    Ok(client.database(database))
}

async fn register_commands(
    context: &Context,
    commands: &[BotCommand],
    guild_id: Option<GuildId>,
) -> Result<(), DiscordError> {
    let commands = builtins::create_application_commands(commands);
    match guild_id {
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
    Ok(())
}
