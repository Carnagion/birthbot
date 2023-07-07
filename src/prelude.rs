//! Re-exports of commonly used items.

use poise::{Command, Context, Framework, FrameworkError};

pub use crate::{
    bot_data::*,
    config::*,
    error::*,
    model::{Birthday, GuildData, MemberData},
    Birthbot,
};

/// Commonly used helper functions and extension traits.
pub mod util;

/// A bot-specific [`Command`] type using [`BotData`] and [`BotError`].
pub type BotCommand = Command<BotData, BotError>;

/// A bot-specific [`Context`] type using [`BotData`] and [`BotError`].
pub type BotContext<'a> = Context<'a, BotData, BotError>;

/// A bot-specific [`Framework`] type using [`BotData`] and [`BotError`].
pub type BotFramework = Framework<BotData, BotError>;

/// A bot-specific [`FrameworkError`] type using [`BotData`] and [`BotError`].
pub type BotFrameworkError<'a> = FrameworkError<'a, BotData, BotError>;

/// A bot-specific [`Result`] type using [`BotError`].
pub type BotResult<T> = Result<T, BotError>;
