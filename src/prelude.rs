use poise::{Command, Context, Framework, FrameworkError};

pub use crate::{
    birthday::Birthday,
    bot_data::*,
    error::*,
    model::{GuildData, MemberData},
};

pub mod util;

pub type BotCommand = Command<BotData, BotError>;

pub type BotContext<'a> = Context<'a, BotData, BotError>;

pub type BotFramework = Framework<BotData, BotError>;

pub type BotFrameworkError<'a> = FrameworkError<'a, BotData, BotError>;

pub type BotResult<T> = Result<T, BotError>;
