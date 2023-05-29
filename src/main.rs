use std::{
    fs::{self, File},
    io::Error as IoError,
    num::ParseIntError,
    path::{Path, PathBuf},
    time::Duration,
};

use clap::Parser;

use log::SetLoggerError;

use mongodm::{mongo::options::ResolverConfig, prelude::*};

use poise::{
    builtins,
    serenity_prelude::{Command, Context, Error as DiscordError, GatewayIntents, GuildId},
    FrameworkOptions,
};

use simplelog::{
    ColorChoice, CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger,
};

use snafu::Snafu;

use birthbot::{commands, prelude::*, tasks};

#[derive(Clone, Debug, Eq, Hash, Parser, PartialEq)]
#[command(author, version, about)]
struct BotConfig {
    /// Discord token.
    #[arg(short, long)]
    token: String,
    /// MongoDB cluster connection URI.
    #[arg(short, long)]
    cluster_uri: String,
    /// Name of the database within the MongoDB cluster.
    #[arg(short, long)]
    database_name: String,
    /// Duration between birthday announcement checks in seconds.
    #[arg(long, value_name = "SECONDS", default_value_t = 900)]
    birthday_check_interval: u64,
    /// Guild ID of testing guild, if any.
    #[arg(long, value_name = "GUILD_ID", value_parser = |value: &str| value.parse().map(GuildId))]
    test_guild_id: Option<GuildId>,
    /// Path to log file.
    #[arg(short, long, value_name = "FILE")]
    log_file: PathBuf,
    /// Path to file listing new updates, if any.
    #[arg(short, long, value_name = "FILE")]
    updates_file: Option<PathBuf>,
}

#[derive(Debug, Snafu)]
enum StartupError {
    #[snafu(context(false))]
    NumParse { source: ParseIntError },
    #[snafu(context(false))]
    Discord { source: DiscordError },
    #[snafu(context(false))]
    Mongodb { source: MongoError },
    #[snafu(context(false))]
    Log { source: SetLoggerError },
    #[snafu(context(false))]
    Io { source: IoError },
}

#[tokio::main]
async fn main() -> Result<(), StartupError> {
    let config = BotConfig::parse();

    init_logger(&config.log_file)?;

    let updates = if let Some(updates_file) = config.updates_file {
        Some(fs::read_to_string(updates_file)?)
    } else {
        None
    };

    BotFramework::builder()
        .token(config.token)
        .intents(GatewayIntents::non_privileged())
        .options(FrameworkOptions {
            commands: vec![commands::birthday()],
            on_error: |error| Box::pin(util::report_framework_error(error)),
            ..Default::default()
        })
        .setup(move |context, _, framework| {
            Box::pin(async move {
                let data = BotData {
                    database: connect_mongodb(&config.cluster_uri, &config.database_name).await?,
                    birthday_check_interval: Duration::from_secs(config.birthday_check_interval),
                };

                register_commands(context, &framework.options().commands, config.test_guild_id)
                    .await?;

                tasks::schedule_birthday_announcer(context.clone(), data.clone())?;

                if let Some(updates) = updates {
                    tasks::announce_updates(context, &data, &updates).await?;
                }

                Ok(data)
            })
        })
        .build()
        .await?
        .start()
        .await?;

    Ok(())
}

fn init_logger(log_path: impl AsRef<Path>) -> Result<(), StartupError> {
    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Warn,
            Config::default(),
            File::create(log_path)?,
        ),
    ])?;
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
