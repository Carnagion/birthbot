use std::{fmt::Debug, time::Duration};

use chrono::OutOfRangeError;

use mongodm::prelude::*;

use poise::serenity_prelude::Error as DiscordError;

use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum BotError {
    #[snafu(context(false))]
    Db {
        source: MongoError,
    },
    #[snafu(context(false))]
    Discord {
        source: DiscordError,
    },
    BsonSer {
        source: BsonSerError,
        debug: String,
    },
    DurationOutOfRange {
        source: OutOfRangeError,
        duration: Duration,
    },
}
