use poise::serenity_prelude as serenity;

use serenity::{ChannelId, Context, CreateMessage};

use tokio::{
    sync::mpsc::{self, Receiver, Sender},
    task,
};

use tracing::error;

use crate::{announcement, error::Result, state::State};

#[tracing::instrument]
pub async fn announce_updates(ctx: Context, data: State, changelog: String) {
    let (tx, rx) = mpsc::channel(100);

    // Spawn a long-running task for posting changelogs
    tokio::spawn(post_changelogs(ctx, changelog, rx));

    if let Err(err) = task::block_in_place(|| queue_changelog_posts(data, tx)) {
        error!("failed to announce changelogs in all guilds: {}", err);
    }
}

#[tracing::instrument]
async fn post_changelogs(ctx: Context, changelog: String, mut rx: Receiver<ChannelId>) {
    let version = concat!(
        "`",
        env!("CARGO_PKG_VERSION_MAJOR"),
        ".",
        env!("CARGO_PKG_VERSION_MINOR"),
        ".",
        env!("CARGO_PKG_VERSION_PATCH"),
        "`",
    );

    while let Some(channel_id) = rx.recv().await {
        let embed = announcement("Update")
            .description("A new update has been released.")
            .field("Version", version, false)
            .field("Changelog", format!("```md\n{}\n```", changelog), false);

        // We continue announcing updates even if it fails in some channels.
        let msg = CreateMessage::new().embed(embed);
        if let Err(err) = channel_id.send_message(&ctx.http, msg).await {
            error!(?err, "failed to announce updates to {}", channel_id);
        }
    }
}

#[tracing::instrument]
fn queue_changelog_posts(data: State, tx: Sender<ChannelId>) -> Result<()> {
    let conn = data.conn.lock().unwrap();
    let mut stmt =
        conn.prepare("select channel_id from announcements where channel_id is not null")?;
    let mut rows = stmt.query(())?;

    while let Some(row) = rows.next()? {
        // NOTE: See the note in `birthday::get`.
        let channel_id = row.get(0).map(|id: i64| ChannelId::new(id as u64))?;

        // NOTE: `Sender::blocking_send` only fails if the corresponding receiver has been closed, at which point
        //       there's no reason to continue since the update announcing task is no longer running.
        let Ok(()) = tx.blocking_send(channel_id) else {
            error!(
                ?channel_id,
                "failed to queue update announcement for {}", channel_id,
            );
            break;
        };
    }

    Ok(())
}
