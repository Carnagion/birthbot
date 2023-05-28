use mongodm::prelude::*;

use poise::futures_util::*;

use crate::prelude::{util::*, *};

#[poise::command(
    slash_command,
    guild_only,
    ephemeral,
    on_error = "util::report_framework_error"
)]
pub async fn list(
    context: BotContext<'_>,
    #[description = "List birthdays in ascending order. Defaults to false."]
    #[flag]
    sorted: bool,
) -> BotResult<()> {
    // Defer the response to allow time for query execution
    context.defer_ephemeral().await?;

    let guild_id = context.guild_id().unwrap(); // PANICS: Will always exist as the command is guild-only

    // Search the database for all birthdays in the guild upto a limit
    let member_repo = context.data().database.repository::<MemberData>();
    let limit = 25; // NOTE: Discord allows up to 25 fields in embeds
    let mut member_data = member_repo
        .find(
            doc! {
                field!(guild_id in MemberData): guild_id.to_bson()?,
            },
            MongoFindOptions::builder()
                .limit(limit as i64)
                .batch_size(limit as u32)
                .build(),
        )
        .await?
        .into_stream()
        .try_collect::<Vec<_>>()
        .await?;

    if member_data.len() == 0 {
        // Report absence of birthdays
        util::embed(&context, true, |embed| {
            embed
                .unchanged()
                .description("There are no birthdays to list.")
        })
        .await
    } else {
        // Display the retrieved birthdays
        util::embed(&context, true, |embed| {
            embed.success().description(format!(
                "{} birthdays were successfully retrieved.",
                member_data.len()
            ));

            // Sort birthdays if requested
            if sorted {
                member_data.sort_unstable_by_key(|member_data| member_data.birthday);
            }

            // Add members and their birthdays to the embed as fields
            member_data.into_iter().fold(embed, |embed, member_data| {
                embed.field(
                    "Birthday",
                    format!("<@{}> - {}", member_data.user_id, member_data.birthday),
                    true,
                )
            })
        })
        .await
    }?;

    Ok(())
}
