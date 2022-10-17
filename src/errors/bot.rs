use std::env::VarError;
use std::num::ParseIntError;

use dotenv::Error as EnvError;

use mongodb::bson::document::ValueAccessError;
use mongodb::error::Error as MongoError;

use serenity::prelude::SerenityError;

#[derive(Debug)]
pub enum BotError {
    UserError(String),
    CommandError(String),
    SerenityError(SerenityError),
    MongoError(MongoError),
    BsonError(ValueAccessError),
    EnvError(EnvError),
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