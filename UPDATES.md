# Slash commands

Renamed `birthday announce` to `birthday channel set`.
- Restricted to users who can manage channels.

Renamed `birthday unannounce` to `birthday channel unset`.
- Restricted to users who can manage channels.

Added `birthday channel get`.

Improved error messages and response embeds.

# Birthday format

Improved time and timezone support.

Added support for human-readable strings when using `birthday set` or displaying birthdays.
- Accepts input such as `17 June 2008` or `17 June 2008, 21:57, +01:00`.

# Birthday announcements

Improved default frequency of birthday checks.
- Checks happen every 15 minutes instead of every 24 hours.

# User data

Improved data model and storage schemas.

# Miscellaneous

Added support for displaying changelogs on updates.
- Announced only if a birthday channel is set.

Changed arguments to be read from the command line rather than environment variables.