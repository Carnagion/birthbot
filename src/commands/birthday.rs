use crate::prelude::*;

mod get;
pub use get::*;

mod set;
pub use set::*;

mod unset;
pub use unset::*;

mod list;
pub use list::*;

mod next;
pub use next::*;

pub mod channel;

#[poise::command(
    slash_command,
    subcommands("get", "set", "unset", "list", "next", "channel::channel"),
    guild_only
)]
pub async fn birthday(_: BotContext<'_>) -> BotResult<()> {
    unreachable!() // PANICS: Will never be reached as the command is slash-only, and parent slash commands cannot be called without subcommands
}
