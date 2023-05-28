use chrono::{prelude::*, Duration};

use log::error;

use mongodm::prelude::*;

use poise::serenity_prelude::{colours::branding::FUCHSIA, *};

use snafu::prelude::*;

use tokio::time;

use crate::prelude::{util::*, *};

pub fn schedule_birthday_announcer(context: Context, data: BotData) -> BotResult<()> {
    let interval = data.birthday_check_interval;
    let half_interval = Duration::from_std(interval)
        .with_context(|_| DurationOutOfRangeSnafu { duration: interval })?
        / 2;

    let mut interval = time::interval(data.birthday_check_interval);

    tokio::spawn(async move {
        loop {
            interval.tick().await;
            if let Err(error) = check_birthdays(&context, &data, half_interval).await {
                error!("Birthday announcing task failed: {}", error);
            }
        }
    });

    Ok(())
}

async fn check_birthdays(
    context: &Context,
    data: &BotData,
    half_interval: Duration,
) -> BotResult<()> {
    let member_repo = data.database.repository::<MemberData>();
    let mut member_data = member_repo.find(None, None).await?;
    while member_data.advance().await? {
        let member_data = member_data.deserialize_current()?;

        // Calculate the range in which a birthday is recognised
        let now = Utc::now(); // NOTE: Done inside loop because time between database requests may be significant

        // Calculate the member's birthday in UTC and with the current year
        let birthday_utc = member_data.birthday.0.with_timezone(&Utc);
        let birthday_now = birthday_utc.with_year(now.year()).unwrap(); // PANICS: Humanity will probably be gone before we reach the max year

        // Announce the member's birthday if it is currently happening within the given interval
        let birthday_range = now - half_interval..now + half_interval;
        if birthday_range.contains(&birthday_now) {
            let age = now.years_since(birthday_utc).unwrap_or(0);
            announce_birthday(context, data, member_data, age).await?;
        }
    }

    Ok(())
}

async fn announce_birthday(
    context: &Context,
    data: &BotData,
    member_data: MemberData,
    age: u32,
) -> BotResult<()> {
    let guild_repo = data.database.repository::<GuildData>();
    let birthday_channel_id = guild_repo
        .find_one(
            doc! {
                field!(guild_id in GuildData): member_data
                .guild_id.to_bson()?,
            },
            None,
        )
        .await?
        .and_then(|guild_data| guild_data.birthday_channel_id);

    if let Some(birthday_channel_id) = birthday_channel_id {
        birthday_channel_id
            .send_message(&context.http, |message| {
                message.embed(|embed| {
                    embed
                        .title("Birthday")
                        .colour(FUCHSIA)
                        .description(format!("It's <@{}>'s birthday!", member_data.user_id))
                        .field("Age", age, true)
                })
            })
            .await?;
    }

    Ok(())
}
