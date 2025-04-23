# Slash commands

- `birthday set` no longer parses birthdays from strings, and instead uses command parameters for the day, month, year, hour, minute, second, and timezone.

- Birthdays are now grouped by month and sorted by day in `birthday list`.

- Birthdays no longer display the time of birth if it is set to the default (i.e. `00:00:00`).

- `birthday channel {set, unset}` now require administrator privileges.

# Birthday announcements

- Birthdays are now checked every hour instead of every 15 minutes.

# User data

- Data is now stored locally on a SQLite database instead of a MongoDB cloud instance.

# Bugfixes

- Birthday announcements no longer stop altogether when a single announcement fails.

- Birthday calculations no longer fail on edge cases like February 29 on a non-leap year.

# Miscellaneous

- Embed titles and errors are now slightly more varied and descriptive.

- Both TOML files and env vars are now supported for configuration.