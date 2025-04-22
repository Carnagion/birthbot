use poise::serenity_prelude as serenity;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("SQLite error: {}", .0)]
    Sqlite(#[from] rusqlite::Error),
    #[error("Discord API error: {}", .0)]
    Discord(#[from] serenity::Error),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
