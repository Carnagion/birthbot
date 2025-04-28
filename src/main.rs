#![deny(rust_2018_idioms)]

use std::{
    fs,
    mem,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use chrono::Datelike;

use figment::{
    Figment,
    providers::{Env, Format, Toml},
};

use poise::{
    CreateReply,
    Framework,
    FrameworkError,
    FrameworkOptions,
    serenity_prelude as serenity,
};

use rusqlite::{Connection, functions::FunctionFlags};

use serde::Deserialize;

use serenity::{
    Client,
    CreateEmbed,
    GatewayIntents,
    colours::{
        branding::{BLURPLE, FUCHSIA},
        css::{DANGER, POSITIVE},
    },
};

use tracing::{error, level_filters::LevelFilter};

use tracing_appender::rolling;

use tracing_subscriber::{Layer as _, fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt};

mod birthday;
use birthday::Birthday;

mod state;
use state::State;

mod error;
use error::{Error, Result};

mod commands;

mod background;
use background::{birthdays::watch_birthdays, changelog::announce_updates};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
struct Config {
    token: String,
    db: PathBuf,
    log_dir: PathBuf,
    changelog_file: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut config = Figment::new()
        .merge(Toml::file("birthbot.toml"))
        .merge(Env::prefixed("BIRTHBOT_"))
        .extract::<Config>()?;

    // Setup logging before anything else so that we can log whatever goes wrong
    let writer = rolling::daily(&config.log_dir, "birthbot.log");
    let layer = Layer::new()
        .with_writer(writer)
        .with_filter(LevelFilter::WARN);
    tracing_subscriber::registry().with(layer).try_init()?;

    let token = mem::take(&mut config.token);

    let framework = Framework::builder()
        .setup(|ctx, _, framework| Box::pin(setup(ctx, framework, config)))
        .options(FrameworkOptions {
            commands: vec![commands::birthday()],
            on_error: |err| {
                Box::pin(async {
                    if let Err(err) = on_error(err).await {
                        error!("failed to reply to interaction: {}", err);
                    }
                })
            },
            ..FrameworkOptions::default()
        })
        .build();

    let mut bot = Client::builder(token, GatewayIntents::non_privileged())
        .framework(framework)
        .await?;

    bot.start().await?;

    Ok(())
}

async fn setup(
    ctx: &serenity::Context,
    framework: &Framework<State, Error>,
    config: Config,
) -> Result<State> {
    let commands = &framework.options().commands;
    poise::builtins::register_globally(ctx, commands).await?;

    let conn = Connection::open(&config.db)?;
    conn.prepare(include_str!("../init/create-birthdays.sql"))?
        .execute(())?;
    conn.prepare(include_str!("../init/create-announcements.sql"))?
        .execute(())?;

    // Register custom functions used for sorting birthdays (see `birthday::list`)
    let flags = FunctionFlags::SQLITE_DETERMINISTIC | FunctionFlags::SQLITE_INNOCUOUS;
    conn.create_scalar_function("day", 1, flags, |ctx| {
        let birthday = ctx.get(0).map(Birthday)?;
        Ok(birthday.0.day())
    })?;
    conn.create_scalar_function("month", 1, flags, |ctx| {
        let birthday = ctx.get(0).map(Birthday)?;
        Ok(birthday.0.month())
    })?;

    let data = State {
        conn: Arc::new(Mutex::new(conn)),
    };

    tokio::spawn(watch_birthdays(ctx.clone(), data.clone()));

    if let Some(changelog_file) = config.changelog_file {
        // PANICS: This realistically won't panic, and I don't want to add a variant to the error enum just for this
        let changelog = fs::read_to_string(changelog_file).unwrap();
        tokio::spawn(announce_updates(ctx.clone(), data.clone(), changelog));
    }

    Ok(data)
}

async fn on_error(err: FrameworkError<'_, State, Error>) -> Result<()> {
    match err {
        FrameworkError::Command { error, ctx, .. } => {
            error!("failed to execute command: {}", error);
            let embed = failure("Command failed").description(format!(
                "An error occurred while handling the command: `{}`",
                error,
            ));
            ctx.send(reply(embed)).await?;
        },
        FrameworkError::CommandPanic { payload, ctx, .. } => {
            error!("command handler panicked: {:?}", payload);
            let embed = failure("Command panicked").description(
                "Something went wrong and the command handler panicked. This indicates a bug in \
                 my code - please [file an issue on GitHub](<https://github.com/Carnagion/birthbot/issues>).",
            );
            ctx.send(reply(embed)).await?;
        },
        FrameworkError::ArgumentParse {
            error, input, ctx, ..
        } => {
            error!("invalid argument: {}", error);
            let embed =
                failure("Invalid input").description("One of the command arguments is invalid.");
            let embed = match input {
                None => embed,
                // NOTE: We place zero-width spaces between codefences in the input to avoid possible injection.
                Some(arg) => embed.field(
                    "Value provided",
                    format!(
                        "```\n{}\n```",
                        arg.replace("```", "\u{200B}`\u{200B}`\u{200B}`"),
                    ),
                    true,
                ),
            };
            ctx.send(reply(embed)).await?;
        },
        FrameworkError::CommandStructureMismatch {
            description, ctx, ..
        } => {
            error!(
                "mismatched command structure for {}: {}",
                ctx.command.name, description,
            );
            let embed = failure("Invalid command").description(
                "That command structure doesn't match what I expected. This indicates that my \
                 latest commands may not have been registered with Discord yet.",
            );
            ctx.send(reply(embed)).await?;
        },
        FrameworkError::CooldownHit {
            remaining_cooldown,
            ctx,
            ..
        } => {
            let embed = failure("Cooldown active").description(format!(
                "You're too fast. Please wait {} before retrying.",
                remaining_cooldown.as_secs(),
            ));
            ctx.send(reply(embed)).await?;
        },
        FrameworkError::MissingBotPermissions {
            missing_permissions,
            ctx,
            ..
        } => {
            let embed = failure("Unauthorised")
                .description("I lack the necessary permissions to execute that command.")
                .field(
                    "Missing permissions",
                    missing_permissions.to_string(),
                    false,
                );
            ctx.send(reply(embed)).await?;
        },
        FrameworkError::MissingUserPermissions {
            missing_permissions,
            ctx,
            ..
        } => {
            let embed = failure("Unauthorised")
                .description("You lack the necessary permissions to issue that command.");
            let embed = match missing_permissions {
                None => embed,
                Some(perms) => embed.field("Missing permissions", perms.to_string(), false),
            };
            ctx.send(reply(embed)).await?;
        },
        FrameworkError::NotAnOwner { ctx, .. } => {
            let embed = failure("Unauthorised").description("Only owners can use that command.");
            ctx.send(reply(embed)).await?;
        },
        FrameworkError::GuildOnly { ctx, .. } => {
            let embed = failure("Invalid context")
                .description("You can only use that command in a guild (i.e. server).");
            ctx.send(reply(embed)).await?;
        },
        FrameworkError::DmOnly { ctx, .. } => {
            let embed =
                failure("Invalid context").description("You can only use that command in DMs.");
            ctx.send(reply(embed)).await?;
        },
        FrameworkError::NsfwOnly { ctx, .. } => {
            let embed = failure("Invalid context")
                .description("You can only use that command in NSFW channels (:flushed:).");
            ctx.send(reply(embed)).await?;
        },
        error => error!("error: {:?}", error),
    }
    Ok(())
}

fn success(title: &str) -> CreateEmbed {
    CreateEmbed::default().colour(POSITIVE).title(title)
}

fn neutral(title: &str) -> CreateEmbed {
    CreateEmbed::default().colour(BLURPLE).title(title)
}

fn failure(title: &str) -> CreateEmbed {
    CreateEmbed::default().colour(DANGER).title(title)
}

fn announcement(title: &str) -> CreateEmbed {
    CreateEmbed::default().colour(FUCHSIA).title(title)
}

fn reply(embed: CreateEmbed) -> CreateReply {
    CreateReply::default().ephemeral(true).embed(embed)
}
