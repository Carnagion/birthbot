use std::{
    fs::{self, File},
    path::Path,
};

use anyhow::Result as AnyResult;

use simplelog::{
    ColorChoice, CombinedLogger, Config as LoggerConfig, LevelFilter, TermLogger, TerminalMode,
    WriteLogger,
};

use birthbot::prelude::*;

#[tokio::main]
async fn main() -> AnyResult<()> {
    // Deserialize config file
    let config_string = fs::read_to_string("birthbot.toml")?;
    let config = toml::from_str::<BirthbotConfig>(&config_string)?;

    setup_logger(&config.log_path)?;

    // Create and start bot
    Birthbot::new(config).await?.start().await?;

    Ok(())
}

fn setup_logger(log_path: impl AsRef<Path>) -> AnyResult<()> {
    CombinedLogger::init(vec![
        // Log most events to stdout
        TermLogger::new(
            LevelFilter::Info,
            LoggerConfig::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        // Log warnings and errors to a log file
        WriteLogger::new(
            LevelFilter::Warn,
            LoggerConfig::default(),
            File::create(log_path)?,
        ),
    ])?;

    Ok(())
}
