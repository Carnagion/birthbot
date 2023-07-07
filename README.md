# Birthbot

**Birthbot** is an open-source and ad-free Discord bot for keeping track of birthdays.
[Add it to your server](https://discord.com/api/oauth2/authorize?client_id=1031249634020044860&permissions=2147485696&scope=bot%20applications.commands).

# Commands

**Birthbot** recognises the following slash commands:
- `birthday get` - Get a user's birthday
- `birthday set` - Set your birthday
- `birthday unset` - Remove your birthday
- `birthday list` - List all birthdays
- `birthday next` - List upcoming birthdays
- `birthday channel get` - Get the birthday announcement channel
- `birthday channel set` - Set the birthday announcement channel
- `birthday channel unset` - Remove the birthday announcement channel
- `birthday help` - Display help and information on how to use the commands

**Birthbot** regularly scans its database for birthdays occurring around the current time, and announces them in the relevant guilds if birthday announcement channels have been provided.

# Data

**Birthbot** only stores the minimum user and guild data required to work with and announce birthdays:
- `birthday set` stores your user ID, guild ID, and the birthday you provide
- `birthday unset` deletes the above
- `birthday channel set` stores your guild ID and the birthday channel ID you provide
- `birthday channel unset` deletes the birthday channel ID

# Credits

**Birthbot**'s icon was taken from [Flaticon](https://www.flaticon.com/free-icons/birthday-cake).