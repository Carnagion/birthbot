use std::time::Duration;

use mongodm::prelude::*;

/// State required by bot commands and long-running tasks.
#[derive(Clone, Debug)]
pub struct BotData {
    /// The database storing all data queried and updated by bot commands.
    pub database: MongoDatabase,
    /// Duration between checks for announcing birthdays.
    pub birthday_check_interval: Duration,
}
