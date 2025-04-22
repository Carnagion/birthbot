use crate::{
    error::{Error, Result},
    state::State,
};

type Context<'a> = poise::Context<'a, State, Error>;

pub mod birthday;

#[poise::command(
    slash_command,
    subcommands(
        "birthday::get",
        "birthday::set",
        "birthday::unset",
        "birthday::list",
        "birthday::next",
        "birthday::channel",
    )
)]
pub async fn birthday(_: Context<'_>) -> Result<()> {
    Ok(())
}
