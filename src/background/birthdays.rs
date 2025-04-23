use chrono::{Months, TimeDelta, Utc};

use poise::serenity_prelude as serenity;

use serenity::{ChannelId, Context, CreateMessage, GuildId, UserId};

use tokio::{
    sync::mpsc::{self, Receiver, Sender},
    task,
    time,
};

use tracing::error;

use crate::{announcement, birthday::Birthday, error::Result, state::State};

const INTERVAL: TimeDelta = TimeDelta::hours(12);

#[tracing::instrument]
pub async fn watch_birthdays(ctx: Context, data: State) {
    // Spawn a long-running task for announcing birthdays found by the birthday-checking task
    let (tx, rx) = mpsc::channel(100);
    tokio::spawn(announce_birthdays(ctx, rx));

    // PANICS: 1 hour is a valid `std::time::Duration`.
    let mut interval = time::interval(INTERVAL.to_std().unwrap());
    loop {
        // NOTE: The first call to `tick` yields immediately. Also, `Interval` already accounts for time passed
        //       since the previous call, so we don't have to handle it manually.
        interval.tick().await;

        let tx = tx.clone();
        if let Err(err) = task::block_in_place(|| queue_birthday_announcements(&data, tx)) {
            error!("failed to announce all birthdays: {}", err);
        }
    }
}

#[derive(Debug)]
struct Announcement {
    user_id: UserId,
    guild_id: GuildId,
    channel_id: ChannelId,
    birthday: Birthday,
}

#[tracing::instrument]
async fn announce_birthdays(ctx: Context, mut rx: Receiver<Announcement>) {
    while let Some(ann) = rx.recv().await {
        let Announcement {
            user_id,
            guild_id,
            channel_id,
            birthday,
        } = ann;

        let now = Utc::now().fixed_offset();
        let age = now.years_since(birthday.0).unwrap(); // PANICS: Future dates are rejected when setting birthdays

        let embed = announcement("Happy birthday!")
            .description(format!("It's <@{}>'s birthday! :partying_face:", user_id))
            .field("Age", age.to_string(), true);

        let message = CreateMessage::default().embed(embed);

        // We continue announcing other birthdays even if some of them fail to be announced.
        if let Err(err) = channel_id.send_message(&ctx, message).await {
            error!(
                ?err,
                ?user_id,
                ?guild_id,
                "failed to send birthday announcement to {}",
                channel_id,
            );
        }
    }
}

#[tracing::instrument]
fn queue_birthday_announcements(data: &State, tx: Sender<Announcement>) -> Result<()> {
    // NOTE: If we calculate the interval inside the loop, it's entirely possible (although unlikely) for
    //       enough time to pass during an iteration that the next iteration's interval misses out on a
    //       birthday which just happened to be during that time spent in the first iteration. As a result,
    //       it's better to fix the interval before we enter the loop and then make up for any time spent
    //       in the loop by waiting less until the next birthday announcement check.
    let now = Utc::now().fixed_offset();
    let interval = now - INTERVAL..now;

    let conn = data.conn.lock().unwrap();
    let mut stmt = conn.prepare("select user_id, guild_id, birthday from birthdays")?;
    let mut rows = stmt.query(())?;

    while let Some(row) = rows.next()? {
        // NOTE: See the note in `birthday::get`.
        let user_id = row.get(0).map(|id: i64| UserId::new(id as u64))?;
        let guild_id = row.get(1).map(|id: i64| GuildId::new(id as u64))?;
        let birthday = row.get(2).map(Birthday)?;

        // NOTE: We can't just use `birthday.0.with_year(now.year())` due to edge cases that might create
        //       invalid dates, such as Feb 29 in a non-leap year. Instead, we figure out how many years
        //       have passed since the date of birth, then add that many years to it. This produces a date
        //       that is either in the year before the current date, or in the same year as the current date.
        //       In the former case, the user has not celebrated their birthday this year, while in the latter
        //       case, the user's birthday has already passed this year.
        let years = now.years_since(birthday.0).unwrap(); // PANICS: Birthdays are always in the past
        let last_birthday = birthday
            .0
            .checked_add_months(Months::new(years * 12))
            .unwrap();
        if !interval.contains(&last_birthday) {
            continue;
        }

        let channel_id = conn
            .prepare(
                "select channel_id from announcements where guild_id = ?1 and channel_id is not \
                 null",
            )?
            .query((guild_id.get() as i64,))? // NOTE: See the note in `birthday::get`.
            .next()?
            .map(|row| row.get(0).map(|id: i64| ChannelId::new(id as u64))) // NOTE: See the note in `birthday::get`.
            .transpose()?;

        let Some(channel_id) = channel_id else {
            continue;
        };

        let ann = Announcement {
            user_id,
            guild_id,
            channel_id,
            birthday,
        };

        // NOTE: `Sender::blocking_send` only fails if the corresponding receiver has been closed, at which point
        //       there's no reason to continue checking for birthdays since we can't announce them anyways.
        let Ok(()) = tx.blocking_send(ann) else {
            error!(
                ?channel_id,
                ?birthday,
                "failed to queue birthday announcement for {} in {}",
                user_id,
                guild_id,
            );
            break;
        };
    }

    Ok(())
}
