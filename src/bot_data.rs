use std::time::Duration;

use mongodm::prelude::*;

#[derive(Clone, Debug)]
pub struct BotData {
    pub database: MongoDatabase,
    pub birthday_check_interval: Duration,
}
