use std::env::VarError;
use std::num::ParseIntError;

use dotenv;
use dotenv::Error as DotenvError;

use mongodb::bson::document::ValueAccessError;
use mongodb::error::Error as MongoDbError;

use serenity::prelude::SerenityError;

#[derive(Debug)]
pub enum BotError {
    CommandError(String),
    DotenvError(DotenvError),
    EnvVarError(VarError),
    MongoDbError(MongoDbError),
    IntParseError(ParseIntError),
    SerenityError(SerenityError),
    BsonValueError(ValueAccessError),
}

impl From<DotenvError> for BotError {
    fn from(error: DotenvError) -> Self {
        BotError::DotenvError(error)
    }
}

impl From<MongoDbError> for BotError {
    fn from(error: MongoDbError) -> Self {
        BotError::MongoDbError(error)
    }
}

impl From<ParseIntError> for BotError {
    fn from(error: ParseIntError) -> Self {
        BotError::IntParseError(error)
    }
}

impl From<SerenityError> for BotError {
    fn from(error: SerenityError) -> Self {
        BotError::SerenityError(error)
    }
}

impl From<ValueAccessError> for BotError {
    fn from(error: ValueAccessError) -> Self {
        BotError::BsonValueError(error)
    }
}

impl From<VarError> for BotError {
    fn from(error: VarError) -> Self {
        BotError::EnvVarError(error)
    }
}