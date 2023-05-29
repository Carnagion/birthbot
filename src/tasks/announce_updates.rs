use mongodm::prelude::*;

use poise::serenity_prelude::{colours::branding::FUCHSIA, *};

use version::version;

use crate::prelude::*;

pub async fn announce_updates(context: &Context, data: &BotData, updates: &str) -> BotResult<()> {
    let guild_repo = data.database.repository::<GuildData>();
    let mut guild_data = guild_repo.find(None, None).await?;
    while guild_data.advance().await? {
        let guild_data = guild_data.deserialize_current()?;
        announce_update(context, guild_data, updates).await?;
    }

    Ok(())
}

async fn announce_update(context: &Context, guild_data: GuildData, updates: &str) -> BotResult<()> {
    if let Some(birthday_channel_id) = guild_data.birthday_channel_id {
        birthday_channel_id
            .send_message(&context.http, |message| {
                message.embed(|embed| {
                    embed
                        .title("Update")
                        .color(FUCHSIA)
                        .description("A new update has been released.")
                        .field("Version", version!(), true)
                        .field("Changelog", updates, false)
                })
            })
            .await?;
    }

    Ok(())
}
