//! Generates and handles the `birthday next` sub-command.

use chrono::Datelike;
use chrono::DateTime;
use chrono::FixedOffset;
use chrono::Utc;

use serenity::builder::CreateApplicationCommandOption;
use serenity::builder::CreateEmbed;
use serenity::model::application::command::CommandOptionType;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::application::interaction::application_command::CommandDataOption;
use serenity::prelude::Context;
use serenity::utils::Colour;

use crate::BotError;

/// Generates the `birthday next` sub-command.
pub fn create_birthday_next_subcommand(subcommmand: &mut CreateApplicationCommandOption) -> &mut CreateApplicationCommandOption {
    subcommmand
        .kind(CommandOptionType::SubCommand)
        .name("next")
        .description("Retrieve incoming birthdays.")
        .create_sub_option(|option| option
            .kind(CommandOptionType::Integer)
            .name("times")
            .description("How many incoming birthdays to retrieve")
            .required(false)
            .min_int_value(1)
            .max_int_value(25)) // Discord embeds do not display more than 25 fields
}

/// Handles the `birthday list` sub-command.
///
/// # Errors
/// A [BotError] is returned in situations including but not limited to:
/// - The sub-command option is not resolved or has an invalid value
/// - There was an error connecting to or querying the database
/// - There was an error responding to the command
pub async fn handle_birthday_next_subcommand(subcommand: &CommandDataOption, command: &ApplicationCommandInteraction, context: &Context) -> Result<(), BotError> {
    let times = *require_command_simple_option!(subcommand.options.get(0), Integer, "times", 1)? as usize;
    let guild = command.guild_id
        .ok_or(BotError::UserError(String::from("This command can only be performed in a guild.")))?;
    // Retrieve and sort birthdays
    let mut birthdays = super::get_all_birthdays(guild).await?;
    birthdays.sort_by(|(_, left), (_, right)| left.cmp(right));
    match birthdays.len() {
        0 => command_error!("There are no birthdays to list.", command, context),
        _ => command_response!(command, context, |data| data
            .ephemeral(true)
            .embed(|embed| birthday_next_embed(embed, &mut birthdays, times))),
    }
}

fn birthday_next_embed<'a>(embed: &'a mut CreateEmbed, birthdays: &mut Vec<(i64, DateTime<FixedOffset>)>, times: usize) -> &'a mut CreateEmbed {
    let description = if times == 1 {
        String::from("The next birthday was successfully retrieved.")
    } else {
        format!("The next {} birthdays were successfully retrieved.", times)
    };
    embed
        .title("Success")
        .description(description)
        .colour(Colour::from_rgb(87, 242, 135));
    // Find index of first birthday that comes after current day
    let now = Utc::now();
    let index = birthdays
        .iter()
        .map(|(user, birth)| (user, birth
            .with_timezone(&Utc)
            .with_year(now.year())
            .unwrap()))
        .position(|(_, birth)| birth > now);
    let advancement = match index {
        None => 0,
        Some(index) => index + 1,
    };
    // Make an infinite iterator and take the required amount
    birthdays
        .iter()
        .cycle()
        .skip(advancement)
        .take(times)
        .fold(embed, |embed, (user, birth)| embed.field("Birthday", format!("<@{}> ({})", user, birth.date()), true))
}