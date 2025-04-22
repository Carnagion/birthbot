use poise::serenity_prelude as serenity;

use serenity::{Channel, ChannelId};

use tokio::task;

use tracing::warn;

use crate::{
    commands::Context,
    error::{Error, Result},
    neutral,
    reply,
    success,
};

#[poise::command(slash_command, guild_only, ephemeral)]
pub async fn get(ctx: Context<'_>) -> Result<()> {
    // Defer response to allow time for executing the query
    ctx.defer_ephemeral().await?;

    let guild_id = ctx.guild_id().unwrap(); // PANICS: Always exists as the command is guild-only

    let channel_id = task::block_in_place(|| {
        let conn = ctx.data().conn.lock().unwrap();
        let query = "select channel_id from announcements where guild_id = ?1";
        let channel_id = conn
            .prepare(query)?
            // NOTE: See the note in `birthday::get`.
            .query((guild_id.get() as i64,))?
            .next()?
            // NOTE: See the note in `birthday::get`.
            .map(|row| row.get(0).map(|id: i64| ChannelId::new(id as u64)))
            .transpose()?;
        Ok::<_, Error>(channel_id)
    })?;

    let embed = match channel_id {
        Some(channel_id) => success("Channel retrieved").description(format!(
            "Birthdays and updates are announced in <#{}>.",
            channel_id,
        )),
        None => neutral("Channel unavailable")
            .description("A birthday announcement channel hasn't been set yet."),
    };

    ctx.send(reply(embed)).await?;

    Ok(())
}

#[poise::command(
    slash_command,
    guild_only,
    ephemeral,
    required_permissions = "MANAGE_CHANNELS"
)]
pub async fn set(
    ctx: Context<'_>,
    #[description = "The birthday announcement channel."]
    #[channel_types("Text")]
    channel: Channel,
) -> Result<()> {
    // Defer response to allow time for executing the query
    ctx.defer_ephemeral().await?;

    let channel_id = channel.id();
    let guild_id = ctx.guild_id().unwrap(); // PANICS: Always exists as the command is guild-only

    task::block_in_place(|| {
        let conn = ctx.data().conn.lock().unwrap();
        let query = "insert into announcements (guild_id, channel_id) values (?1, ?2) on conflict \
                     (guild_id) do update set channel_id = excluded.channel_id";
        // NOTE: See the note in `birthday::get`.
        conn.execute(query, (guild_id.get() as i64, channel_id.get() as i64))?;
        Ok::<_, Error>(())
    })?;

    let embed = success("Channel updated").description(format!(
        "The birthday announcement channel has been updated to <#{}>.",
        channel_id,
    ));

    ctx.send(reply(embed)).await?;

    Ok(())
}

#[poise::command(
    slash_command,
    guild_only,
    ephemeral,
    required_permissions = "ADMINISTRATOR"
)]
pub async fn unset(ctx: Context<'_>) -> Result<()> {
    // Defer response to allow time for executing the query
    ctx.defer_ephemeral().await?;

    let guild_id = ctx.guild_id().unwrap(); // PANICS: Always exists as the command is guild-only

    let deleted = task::block_in_place(|| {
        let conn = ctx.data().conn.lock().unwrap();
        let query = "delete from announcements where guild_id = ?1";
        // NOTE: See the note in `birthday::get`.
        let affected = conn.execute(query, (guild_id.get() as i64,))?;

        // NOTE: Guild IDs uniquely identify a row, so if more than 1 row was deleted then something has gone wrong.
        if affected > 1 {
            warn!(
                ?guild_id,
                "{} rows affected by `birthday channel unset`", affected,
            );
        }

        Ok::<_, Error>(affected >= 1)
    })?;

    let embed = if deleted {
        success("Channel unset").description("Birthdays are no longer announced in any channel.")
    } else {
        neutral("Channel unavailable")
            .description("A birthday announcement channel hasn't been set yet.")
    };

    ctx.send(reply(embed)).await?;

    Ok(())
}
