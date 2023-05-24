use poise::{Command, Context, Framework};

pub use crate::{
    birthday::Birthday,
    bot_data::*,
    error::*,
    model::{GuildData, MemberData},
};

pub(crate) mod utils;

pub type BotCommand = Command<BotData, BotError>;

pub type BotContext<'a> = Context<'a, BotData, BotError>;

pub type BotFramework = Framework<BotData, BotError>;

pub type BotResult<T> = Result<T, BotError>;
