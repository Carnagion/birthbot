use mongodm::prelude::*;

use crate::prelude::{util::*, *};

#[poise::command(slash_command, guild_only)]
pub async fn unset(context: BotContext<'_>) -> BotResult<()> {
    // Defer the response to allow time for query execution
    context.defer_or_broadcast().await?;

    let user_id = context.author().id;
    let guild_id = context.guild_id().unwrap(); // PANICS: Will always exist as the command is guild-only

    // Delete the member's birthday
    let member_repo = context.data().database.repository::<MemberData>();
    let deleted = member_repo
        .delete_one(
            doc! {
                field!(user_id in MemberData): user_id.to_bson()?,
                field!(guild_id in MemberData): guild_id.to_bson()?,
            },
            None,
        )
        .await?;

    if deleted.deleted_count == 0 {
        // Report the absence of the member's birthday
        util::embed(&context, true, |embed| {
            embed
                .unchanged()
                .description("You haven't set a birthday yet.")
        })
        .await
    } else {
        // Acknowledge deletion of the member's birthday
        util::embed(&context, true, |embed| {
            embed
                .success()
                .description("Your birthday was successfully removed.")
        })
        .await
    }?;

    Ok(())
}
