use mongodm::prelude::*;

use poise::serenity_prelude::*;

use crate::prelude::{util::*, *};

#[poise::command(
    slash_command,
    guild_only,
    ephemeral,
    on_error = "util::report_framework_error"
)]
pub async fn get(
    context: BotContext<'_>,
    #[description = "Whose birthday to retrieve. Defaults to you."] user: Option<User>,
) -> BotResult<()> {
    // Defer the response to allow time for query execution
    context.defer_ephemeral().await?;

    let user_id = user.as_ref().unwrap_or(context.author()).id;
    let guild_id = context.guild_id().unwrap(); // PANICS: Will always exist as the command is guild-only

    // Search the database for the requested member's birthday
    let member_repo = context.data().database.repository::<MemberData>();
    let member_data = member_repo
        .find_one(
            doc! {
                field!(user_id in MemberData): user_id.to_bson()?,
                field!(guild_id in MemberData): guild_id.to_bson()?,
            },
            None,
        )
        .await?;

    match member_data {
        // Display the retrieved birthday
        Some(member_data) => {
            embed(&context, true, |embed| {
                embed
                    .success()
                    .description(if user_id == context.author().id {
                        "Your birthday was successfully retrieved.".to_owned()
                    } else {
                        format!("<@{}>'s birthday was successfully retrieved.", user_id)
                    })
                    .field("Birthday", format!("`{}`", member_data.birthday), true)
            })
            .await
        },
        // Report the absence of a birthday for the requested member
        None => {
            embed(&context, true, |embed| {
                embed
                    .unchanged()
                    .description(if user_id == context.author().id {
                        "You haven't set a birthday yet.".to_owned()
                    } else {
                        format!("<@{}> hasn't set a birthday yet.", user_id)
                    })
            })
            .await
        },
    }?;

    Ok(())
}
