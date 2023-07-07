use mongodm::prelude::*;

use poise::serenity_prelude as serenity;

use serenity::colours::branding::BLURPLE;

use crate::prelude::{util::*, *};

/// Show help text.
#[poise::command(slash_command, ephemeral, on_error = "util::report_framework_error")]
pub async fn help(context: BotContext<'_>) -> BotResult<()> {
    // NOTE: Yes I know multi-line strings like these are cursed
    embed(&context, false, |embed| {
        embed
            .title("Help")
            .colour(BLURPLE)
            .description("Here is a list of available commands.")
            .field(
                "Get a user's birthday",
                "\
```less
/birthday get [USER]
```
Defaults to your birthday if no user is specified.",
                false,
            )
            .field(
                "Set your birthday",
                "\
```less
/birthday set [BIRTHDAY]
```
Birthdays must be in RFC-3339 or `[DAY] [MONTH] [YEAR]` format, with an optional time and timezone. \
For example:
- `1 November 2007`
- `19 June 2002, 01:13`
- `23 June 1996, 14:35, +09:00`
- `2007-11-01`
- `2002-07-19T01:13`
- `1996-06-23T14:35+09:00`",
                false,
            )
            .field(
                "Remove your birthday",
                "\
```less
/birthday unset
```",
                false,
            )
            .field(
                "List all birthdays",
                "\
```less
/birthday list [SORTED]
```
Displays birthdays in unsorted order if `sorted` is not specified.",
                false,
            )
            .field(
                "List upcoming birthdays",
                "\
```less
/birthday next [LIMIT]
```
Displays the first upcoming birthday if `limit` is not specified.",
                false,
            )
            .field(
                "Get the birthday channel",
                "\
```less
/birthday channel get
```",
                false,
            )
            .field(
                "Set the birthday channel",
                "\
```less
/birthday channel set [CHANNEL]
```",
                false,
            )
            .field(
                "Remove the birthday channel",
                "\
```less
/birthday channel unset
```",
                false,
            )
            .field(
                "Show help",
                "\
```less
/birthday help
```",
                false,
            )
    })
    .await?;

    Ok(())
}
