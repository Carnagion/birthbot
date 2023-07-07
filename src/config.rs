//! Configuration types.

use std::path::PathBuf;

use poise::serenity_prelude as serenity;

use serde::{Deserialize, Serialize};

use serenity::*;

/// Secrets and other configuration data loaded through a `birthbot.toml` file.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct BirthbotConfig {
    /// The bot's Discord token.
    pub bot_token: String,
    /// The duration between birthday announcement checks, in seconds.
    pub test_guild_id: Option<GuildId>,
    /// The guild ID of the testing guild, if any.
    pub birthday_check_interval: Option<u64>,
    /// The path to a file listing new updates, if any.
    pub updates_path: Option<PathBuf>,
    /// The log file path.
    pub log_path: PathBuf,
    /// MongoDB-related configuration data.
    pub database: DatabaseConfig,
}

/// MongoDB-related secrets and configuration data.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct DatabaseConfig {
    /// The bot's MongoDB cluster connection URI.
    pub cluster_uri: String,
    /// The name of the database within the MongoDB cluster.
    pub name: String,
}
