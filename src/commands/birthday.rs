use std::{fmt::Write, mem};

use chrono::{Datelike, FixedOffset, Months, NaiveDate, NaiveTime, Offset, Utc};

use poise::serenity_prelude as serenity;

use serenity::{Member, UserId};

use tokio::task;

use tracing::warn;

use crate::{
    birthday::{Birthday, Month},
    error::{Error, Result},
    failure,
    neutral,
    reply,
    success,
};

use super::Context;

pub mod channel;

#[poise::command(slash_command, guild_only, ephemeral)]
#[tracing::instrument]
pub async fn get(
    ctx: Context<'_>,
    #[description = "Whose birthday to retrieve. Defaults to you."] member: Option<Member>,
) -> Result<()> {
    // Defer response to allow time for executing the query
    ctx.defer_ephemeral().await?;

    let user = match &member {
        Some(member) => &member.user,
        None => ctx.author(),
    };

    let user_id = user.id;
    let guild_id = ctx.guild_id().unwrap(); // PANICS: Always exists as the command is guild-only

    let birthday = task::block_in_place(|| {
        let conn = ctx.data().conn.lock().unwrap();
        let query = "select birthday from birthdays where user_id = ?1 and guild_id = ?2";
        let birthday = conn
            .prepare(query)?
            // NOTE: We need to cast the Discord IDs here since SQLite stores integers as `i64`, and
            //       will throw an error if `i64::try_from` fails. However, casting all `u64` values
            //       to `i64` during insertion and all `i64` values to `u64` during retrieval will
            //       produce the same results while also being infallible.
            .query((user_id.get() as i64, guild_id.get() as i64))?
            .next()?
            .map(|row| row.get(0).map(Birthday))
            .transpose()?;
        Ok::<_, Error>(birthday)
    })?;

    let embed = match birthday {
        Some(birthday) => {
            let now = Utc::now().fixed_offset();
            let age = now.years_since(birthday.0).unwrap(); // PANICS: Future dates are rejected when setting birthdays

            // NOTE: We check if the user ID is the same as the author's ID rather than checking if `member` is `Some`
            //       because this way we can display the correct message even if the user passes in their own ID as the
            //       command argument.
            success("Birthday retrieved")
                .description(if user_id == ctx.author().id {
                    format!("You were born on `{}`.", birthday)
                } else {
                    format!("<@{}> was born on `{}`.", user_id, birthday)
                })
                .field("Age", age.to_string(), true)
        },
        // NOTE: See above.
        None => neutral("Birthday unavailable").description(if user_id == ctx.author().id {
            "You haven't set a birthday yet. Use `/birthday help` for information.".to_owned()
        } else {
            format!("<@{}> hasn't set a birthday yet.", user_id)
        }),
    };

    ctx.send(reply(embed)).await?;

    Ok(())
}

#[poise::command(slash_command, guild_only, ephemeral)]
#[tracing::instrument]
pub async fn set(
    ctx: Context<'_>,
    #[description = "The day you were born on."]
    #[min = 1]
    #[max = 31]
    day: u8,
    #[description = "The month you were born in."] month: Month,
    #[description = "The year you were born in."] year: i32,
    #[description = "The hour you were born in. Defaults to 0."]
    #[max = 23]
    hour: Option<u8>,
    #[description = "The minute you were born in. Defaults to 0."]
    #[max = 59]
    minute: Option<u8>,
    #[description = "The second you were born in. Defaults to 0."]
    #[max = 59]
    second: Option<u8>,
    #[description = "The timezone you were born in. Accepts offsets as `+00:00` or `-00:00`. \
                     Defaults to `+00:00` (UTC)."]
    timezone: Option<FixedOffset>,
) -> Result<()> {
    // Defer response to allow time for executing the query
    ctx.defer_ephemeral().await?;

    // Ensure the date is valid
    let month = month as u32;
    let Some(date) = NaiveDate::from_ymd_opt(year, month, day.into()) else {
        let embed = failure("Invalid birthday")
            .description("That's not a valid year-month-day combination.")
            .field("Year", year.to_string(), true)
            .field("Month", month.to_string(), true)
            .field("Day", day.to_string(), true);
        ctx.send(reply(embed)).await?;
        return Ok(());
    };

    // Ensure the time is valid, defaulting to 00:00:00 if not provided
    let hour = hour.map(u32::from).unwrap_or(0);
    let minute = minute.map(u32::from).unwrap_or(0);
    let second = second.map(u32::from).unwrap_or(0);
    let Some(time) = NaiveTime::from_hms_opt(hour, minute, second) else {
        let embed = failure("Invalid birthday")
            .description("That's not a valid hour-minute-second combination.")
            .field("Hour", hour.to_string(), true)
            .field("Minute", minute.to_string(), true)
            .field("Second", second.to_string(), true);
        ctx.send(reply(embed)).await?;
        return Ok(());
    };

    let timezone = timezone.unwrap_or(Utc.fix());

    // PANICS: I don't want to think about the edge cases for this. If it fails it fails.
    let birthday = Birthday(date.and_time(time).and_local_timezone(timezone).unwrap());

    // Ensure the birthday is not in a future date because that would be silly
    let now = Utc::now().fixed_offset();
    if birthday.0 >= now {
        let embed = failure("Invalid birthday")
            .description("Time travel doesn't exist yet, so your birthday can't be in the future.")
            .field("Provided birthday", format!("```\n{}\n```", birthday), true);
        ctx.send(reply(embed)).await?;
        return Ok(());
    }

    let user_id = ctx.author().id;
    let guild_id = ctx.guild_id().unwrap(); // PANICS: Always exists as the command is guild-only

    task::block_in_place(|| {
        let conn = ctx.data().conn.lock().unwrap();
        let query = "insert into birthdays (user_id, guild_id, birthday) values (?1, ?2, ?3) on \
                     conflict (user_id, guild_id) do update set birthday = excluded.birthday";
        conn.execute(
            query,
            // NOTE: See the note in `birthday::get`.
            (user_id.get() as i64, guild_id.get() as i64, birthday.0),
        )?;
        Ok::<_, Error>(())
    })?;

    let age = now.years_since(birthday.0).unwrap(); // PANICS: Future dates are rejected above
    let embed = success("Birthday updated")
        .description(format!("Your birthday has been updated to `{}`.", birthday,))
        .field("Age", age.to_string(), true);

    ctx.send(reply(embed)).await?;

    Ok(())
}

#[poise::command(slash_command, guild_only, ephemeral)]
#[tracing::instrument]
pub async fn unset(ctx: Context<'_>) -> Result<()> {
    // Defer response to allow time for executing the query
    ctx.defer_ephemeral().await?;

    let user_id = ctx.author().id;
    let guild_id = ctx.guild_id().unwrap(); // PANICS: Always exists as the command is guild-only

    let deleted = task::block_in_place(|| {
        let conn = ctx.data().conn.lock().unwrap();
        let query = "delete from birthdays where user_id = ?1 and guild_id = ?2";
        // NOTE: See the note in `birthday::get`.
        let affected = conn.execute(query, (user_id.get() as i64, guild_id.get() as i64))?;

        // NOTE: User IDs and guild IDs together uniquely identify a single entry, so if more than 1 row was deleted then
        //       something has gone wrong.
        if affected > 1 {
            warn!(
                ?user_id,
                ?guild_id,
                "{} rows affected by `birthday unset`",
                affected,
            );
        }

        Ok::<_, Error>(affected >= 1)
    })?;

    ctx.send(reply(if deleted {
        success("Birthday unset").description("Your birthday was removed.")
    } else {
        neutral("Birthday unavailable").description("You haven't set a birthday yet.")
    }))
    .await?;

    Ok(())
}

#[poise::command(slash_command, guild_only, ephemeral)]
#[tracing::instrument]
pub async fn list(ctx: Context<'_>) -> Result<()> {
    // Defer response to allow time for executing the query
    ctx.defer_ephemeral().await?;

    let guild_id = ctx.guild_id().unwrap(); // PANICS: Always exists as the command is guild-only

    // TODO: Use pagination to allow displaying more birthdays overall
    let fields = task::block_in_place(|| {
        let conn = ctx.data().conn.lock().unwrap();
        let query = "select user_id, birthday from birthdays where guild_id = ?1 order by \
                     month(birthday), day(birthday)";
        let mut stmt = conn.prepare(query)?;
        let mut rows = stmt.query((guild_id.get() as i64,))?; // NOTE: See the note in `birthday::get`.

        let mut fields = Vec::new();
        let mut month = 0;
        let mut field = String::new();
        while let Some(row) = rows.next()? {
            // NOTE: See the note in `birthday::get`.
            let user_id = row.get(0).map(|id: i64| UserId::new(id as u64))?;
            let birthday = row.get(1).map(Birthday)?;

            if month == 0 {
                month = birthday.0.month();
            }

            if birthday.0.month() != month {
                let month_name = chrono::Month::try_from(month as u8).unwrap().name();
                fields.push((month_name, mem::take(&mut field), false));

                month = birthday.0.month();
                field.clear();
            }

            writeln!(&mut field, "<@{}> (`{}`)", user_id, birthday).unwrap();
        }

        let month_name = chrono::Month::try_from(month as u8).unwrap().name();
        fields.push((month_name, field, false));

        Ok::<_, Error>(fields)
    })?;

    let embed = match fields.len() {
        0 => neutral("Birthdays unavailable").description("No birthdays have been set yet."),
        1 => success("Birthdays retrieved").description("Showing 1 birthday."),
        n => success("Birthdays retrieved").description(format!("Showing {} birthdays.", n)),
    }
    .fields(fields);

    ctx.send(reply(embed)).await?;

    Ok(())
}

#[poise::command(slash_command, guild_only, ephemeral)]
#[tracing::instrument]
pub async fn next(
    ctx: Context<'_>,
    #[description = "How many upcoming birthdays to display. Defaults to 1."]
    #[min = 1]
    limit: Option<usize>,
) -> Result<()> {
    // Defer response to allow time for executing the query
    ctx.defer_ephemeral().await?;

    let guild_id = ctx.guild_id().unwrap(); // PANICS: Always exists as the command is guild-only

    // TODO: Use pagination to allow displaying more birthdays overall
    let mut upcoming = task::block_in_place(|| {
        let conn = ctx.data().conn.lock().unwrap();
        let query = "select user_id, birthday from birthdays where guild_id = ?1 order by \
                     month(birthday), day(birthday)";
        let mut stmt = conn.prepare(query)?;
        let mut rows = stmt.query((guild_id.get() as i64,))?; // NOTE: See the note in `birthday::get`.

        let now = Utc::now().fixed_offset();
        let mut upcoming = Vec::new();
        while let Some(row) = rows.next()? {
            // NOTE: See the note in `birthday::get`.
            let user_id = row.get(0).map(|id: i64| UserId::new(id as u64))?;
            let birthday = row.get(1).map(Birthday)?;

            // NOTE: See the note in `announce.rs`.
            let years = now.years_since(birthday.0).unwrap(); // PANICS: Birthdays are always in the past
            let last_birthday = birthday
                .0
                .checked_add_months(Months::new(years * 12))
                .unwrap();

            upcoming.push((user_id, birthday, last_birthday));
        }

        Ok::<_, Error>(upcoming)
    })?;

    upcoming.sort_by_key(|(_, _, last_birthday)| *last_birthday);

    let limit = limit.unwrap_or(1);
    let upcoming = upcoming.into_iter().take(limit).fold(
        String::new(),
        |mut field, (user_id, birthday, _)| {
            writeln!(&mut field, "<@{}> (`{}`)", user_id, birthday).unwrap();
            field
        },
    );

    let embed = match upcoming.len() {
        0 => neutral("Birthdays unavailable").description("No birthdays have been set yet."),
        1 => success("Birthdays retrieved").description("Showing 1 birthday."),
        n => success("Birthdays retrieved").description(format!("Showing {} birthdays.", n)),
    }
    .field("Upcoming birthdays", upcoming, false);

    ctx.send(reply(embed)).await?;

    Ok(())
}

#[poise::command(
    slash_command,
    subcommands("channel::get", "channel::set", "channel::unset")
)]
pub async fn channel(_: Context<'_>) -> Result<()> {
    Ok(())
}
