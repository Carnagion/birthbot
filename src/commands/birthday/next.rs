//! Generates and handles the `birthday next` sub-command.

use chrono::Datelike;
use chrono::Utc;

use serenity::builder::CreateApplicationCommandOption;
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
            .required(false))
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
    let birthdays = super::get_all_birthdays(guild).await?;
    match birthdays.get(0) {
        // If vector is empty, there are no birthdays
        None => command_error!("There are no birthdays to list.", command, context),
        // If vector has elements, find next birthdays
        Some(oldest) => {
            if times > birthdays.len() {
                command_error!("Cannot retrieve more incoming birthdays than the total number of birthdays.", command, context)
            } else {
                // Filter the birthdays that come after the current date using the oldest year as the base year
                let base = Utc::now()
                    .with_year(oldest.1
                        .with_timezone(&Utc)
                        .year())
                    .unwrap();
                let after = birthdays
                    .iter()
                    .filter(|(_, birthday)| birthday.with_timezone(&Utc) >= base)
                    .take(times);
                // Different description for one birthday vs many birthdays
                let description = if times == 1 {
                    String::from("The next birthday was successfully retrieved.")
                } else {
                    format!("The next {} birthdays were successfully retrieved.", times)
                };
                // Respond to command
                command_response!(command, context, |data| data
                .ephemeral(true)
                .embed(|embed| {
                    embed
                        .title("Success")
                        .description(description)
                        .colour(Colour::from_rgb(87, 242, 135));
                    let mut taken = 0;
                    for (user, birth) in after {
                        embed.field("Birthday", format!("<@{}> ({})", user, birth.date()), true);
                        taken += 1;
                    }
                    if taken < times {
                        for (user, birth) in birthdays.iter().take(times - taken) {
                            embed.field("Birthday", format!("<@{}> ({})", user, birth.date()), true);
                        }
                    }
                    embed}))
            }
        },
    }
}