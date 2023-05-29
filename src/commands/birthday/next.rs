use chrono::prelude::*;

use mongodm::prelude::*;

use poise::futures_util::*;

use crate::prelude::{util::*, *};

#[poise::command(
    slash_command,
    guild_only,
    ephemeral,
    on_error = "util::report_framework_error"
)]
pub async fn next(
    context: BotContext<'_>,
    #[description = "How many upcoming birthdays to retrieve. Defaults to 1."]
    #[min = 1]
    #[max = 25]
    limit: Option<u8>,
) -> BotResult<()> {
    // Defer the response to allow time for query execution
    context.defer_ephemeral().await?;

    let guild_id = context.guild_id().unwrap(); // PANICS: Will always exist as the command is guild-only

    // Search the database for all birthdays in the guild upto a limit
    let member_repo = context.data().database.repository::<MemberData>();
    let limit = limit.unwrap_or(1);
    let now = Utc::now();
    let mut member_data = member_repo
        .find(
            doc! {
                field!(guild_id in MemberData): guild_id.to_bson()?,
            },
            MongoFindOptions::builder().batch_size(limit as u32).build(),
        )
        .await?
        .into_stream()
        .map_ok(|member_data| {
            (
                member_data,
                member_data
                    .birthday
                    .0
                    .with_timezone(&Utc)
                    .with_year(now.year())
                    .unwrap(), // PANICS: Current year will always be valid
            )
        })
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
            embed.success().description(if member_data.len() == 1 {
                "The next birthday was successfully retrieved."
            } else {
                "The next birthdays were successfully retrieved."
            });

            // Sort birthdays and find the first one that comes after the current day
            member_data.sort_unstable_by_key(|(_, birthday_utc)| *birthday_utc);
            let skip = member_data
                .iter()
                .position(|(_, birthday_utc)| birthday_utc > &now)
                .unwrap_or(0); // If there are no birthdays after the current time this year, then pick the first birthday next year

            // Add members and their birthdays to the embed as fields
            member_data
                .into_iter()
                .cycle()
                .skip(skip)
                .take(limit as usize)
                .map(|(member_data, _)| member_data)
                .fold(embed, |embed, member_data| {
                    // Add the birthday as a field
                    embed.field(
                        "Birthday",
                        format!("`{}` - <@{}>", member_data.birthday, member_data.user_id),
                        true,
                    )
                })
        })
        .await
    }?;

    Ok(())
}
