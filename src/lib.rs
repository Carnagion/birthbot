//! A birthday bot for Discord that is ad-free, easy to use, and stores minimal data.

#![warn(missing_docs)]

/// Re-exports of commonly used items.
pub mod prelude;

pub mod config;

/// Member and guild data models used by the bot.
pub mod model;

/// Commands provided by the bot.
#[allow(missing_docs)]
pub mod commands; // NOTE: Poise does not generate documentation for the commands generated by its macro

/// Long-running tasks provided by the bot.
pub mod tasks;

mod bot_data;

mod error;
