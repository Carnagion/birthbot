use std::env::VarError;
use std::num::ParseIntError;

use dotenv::Error as EnvError;

use mongodb::bson::document::ValueAccessError;
use mongodb::error::Error as MongoError;

use serenity::prelude::SerenityError;

/// An error that can occur during command handling.
#[derive(Debug)]
pub enum BotError {
    /// A user error, such as providing an invalid option value.
    UserError(String),
    /// An error related to the Discord command API.
    CommandError(String),
    /// An error related to the Serenity API.
    SerenityError(SerenityError),
    /// An error related to the MongoDB API.
    MongoError(MongoError),
    /// An error related to the BSON document API.
    BsonError(ValueAccessError),
    /// An error related to environment variables.
    EnvError(EnvError),
    /// An error related to parsing a [String] into a number.
    ParseError(ParseIntError),
}

impl From<SerenityError> for BotError {
    fn from(error: SerenityError) -> Self {
        BotError::SerenityError(error)
    }
}

impl From<MongoError> for BotError {
    fn from(error: MongoError) -> Self {
        BotError::MongoError(error)
    }
}

impl From<ValueAccessError> for BotError {
    fn from(error: ValueAccessError) -> Self {
        BotError::BsonError(error)
    }
}

impl From<EnvError> for BotError {
    fn from(error: EnvError) -> Self {
        BotError::EnvError(error)
    }
}

impl From<VarError> for BotError {
    fn from(error: VarError) -> Self {
        BotError::EnvError(EnvError::EnvVar(error))
    }
}

impl From<ParseIntError> for BotError {
    fn from(error: ParseIntError) -> Self {
        BotError::ParseError(error)
    }
}