use mongodm::prelude::*;

use poise::serenity_prelude::Channel;

use crate::prelude::{utils::*, *};

#[poise::command(slash_command, guild_only)]
pub async fn set(
    context: BotContext<'_>,
    #[description = "The birthday channel."]
    #[channel_types("Text")]
    channel: Channel,
) -> BotResult<()> {
    // Defer the response to allow time for query execution
    context.defer_or_broadcast().await?;

    let guild_id = context.guild_id().unwrap(); // PANICS: Will always exist as the command is guild-only

    // Insert or update the requested guild's birthday channel
    let guild_repo = context.data().database.repository::<GuildData>();
    let channel_id = guild_repo
        .find_one_and_update(
            doc! {
                field!(guild_id in GuildData): guild_id.to_bson()?,
            },
            doc! {
                Set: {
                    field!(birthday_channel_id in GuildData): channel.id().to_bson()?,
                },
                SetOnInsert: {
                    field!(guild_id in GuildData): guild_id.to_bson()?,
                },
            },
            MongoFindOneAndUpdateOptions::builder().upsert(true).build(),
        )
        .await?
        .and_then(|guild_data| guild_data.birthday_channel_id)
        .unwrap(); // PANICS: Will always exist as the document is upserted

    // Display the updated birthday channel
    utils::embed(&context, false, |embed| {
        embed
            .success()
            .description("The birthday channel was successfully set.")
            .field("Channel", format!("<#{}>", channel_id), true)
    })
    .await?;

    Ok(())
}
