use mongodm::prelude::*;

use crate::prelude::{utils::*, *};

#[poise::command(slash_command, guild_only)]
pub async fn get(context: BotContext<'_>) -> BotResult<()> {
    // Defer the response to allow time for query execution
    context.defer_or_broadcast().await?;

    let guild_id = context.guild_id().unwrap(); // PANICS: Will always exist as the command is guild-only

    // Search the database for the requested guild's birthday channel
    let guild_repo = context.data().database.repository::<GuildData>();
    let channel_id = guild_repo
        .find_one(
            doc! {
                field!(guild_id in GuildData): guild_id.to_bson()?,
            },
            None,
        )
        .await?
        .and_then(|guild_data| guild_data.birthday_channel_id);

    match channel_id {
        // Display the retrieved birthday channel
        Some(channel_id) => {
            utils::embed(&context, true, |embed| {
                embed
                    .success()
                    .description("The birthday channel was successfully retrieved.")
                    .field("Channel", format!("<#{}>", channel_id), true)
            })
            .await
        },
        // Report the absence of a birthday channel for the guild
        None => {
            utils::embed(&context, true, |embed| {
                embed
                    .unchanged()
                    .description("The birthday channel hasn't been set yet.")
            })
            .await
        },
    }?;

    Ok(())
}
