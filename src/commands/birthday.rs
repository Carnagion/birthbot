//! Birthday-related commands.

use crate::prelude::*;

mod help;
pub use help::*;

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
pub use channel::channel;

/// Parent command for all birthday-related subcommands.
///
/// This command cannot actually be called by itself - it requires a subcommand.
#[poise::command(
    slash_command,
    subcommands("help", "get", "set", "unset", "list", "next", "channel"),
    guild_only,
    on_error = "util::report_framework_error"
)]
pub async fn birthday(_: BotContext<'_>) -> BotResult<()> {
    unreachable!() // PANICS: Will never be reached as the command is slash-only, and parent slash commands cannot be called without subcommands
}
