use mongodm::prelude::*;

use crate::prelude::{util::*, *};

#[poise::command(slash_command, guild_only, on_error = "util::report_framework_error")]
pub async fn unset(context: BotContext<'_>) -> BotResult<()> {
    // Defer the response to allow time for query execution
    context.defer_or_broadcast().await?;

    let guild_id = context.guild_id().unwrap(); // PANICS: Will always exist as the command is guild-only

    // Remove the guild's birthday channel
    let guild_repo = context.data().database.repository::<GuildData>();
    let updated = guild_repo
        .update_one(
            doc! {
                field!(guild_id in GuildData): guild_id.to_bson()?,
            },
            doc! {
                Unset: {
                    field!(birthday_channel_id in GuildData): null,
                },
            },
            None,
        )
        .await?;

    if updated.modified_count == 0 {
        // Report the absence of a birthday channel for the guild
        util::embed(&context, true, |embed| {
            embed
                .unchanged()
                .description("The birthday channel hasn't been set yet.")
        })
        .await
    } else {
        // Acknowledge deletion of the birthday channel
        util::embed(&context, false, |embed| {
            embed
                .success()
                .description("The birthday channel was successfully removed.")
        })
        .await
    }?;

    Ok(())
}
