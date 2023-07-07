use std::{fmt::Debug, io::Error as IoError, path::PathBuf, time::Duration};

use chrono::OutOfRangeError;

use mongodm::prelude::*;

use poise::serenity_prelude as serenity;

use serenity::*;

use snafu::prelude::*;

/// Possible errors that could be produced during execution of a bot command or long-running task.
#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum BotError {
    /// A database operation failed.
    #[snafu(context(false), display("database error: {}", source))]
    Db {
        /// The underlying source of the database error.
        source: MongoError,
    },
    /// An operatioon related to Discord failed.
    #[snafu(context(false), display("discord error: {}", source))]
    Discord {
        /// The underlying source of the Discord error.
        source: SerenityError,
    },
    /// A value could not be serialized to BSON.
    #[snafu(display("could not serialize {} to BSON: {}", debug, source))]
    BsonSer {
        /// The underlying source of the serialization error.
        source: BsonSerError,
        /// A debug representation of the value being serialized.
        debug: String,
    },
    /// A [`Duration`] was out of an expected range.
    #[snafu(display("a duration {:?} was out of range: {}", duration, source))]
    DurationOutOfRange {
        /// The underlying soruce of the out-of-range error.
        source: OutOfRangeError,
        /// The duration in question.
        duration: Duration,
    },
    /// A file could not be opened or read.
    #[snafu(display("failed to open or read data from {}: {}", path.display(), source))]
    File {
        /// The underlying source of the file error.
        source: IoError,
        /// The path of the file trying to be opened or read.
        path: PathBuf,
    },
}
