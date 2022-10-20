# Birthbot

**Birthbot** is a Discord bot for keeping track of birthdays.

# Setup

- ## Environment

  **Birthbot** expects a `.env` file in its executing directory (or its parent(s)).
  This must contain the following information:
  - A `TOKEN` key, with **Birthbot**'s secret token
  - A `CLUSTER` key, with the URI of the MongoDB cluster to use (usually in the format `mongodb+srv://USERNAME:PASSWORD@CLUSTER.CODE.mongodb.net/?retryWrites=true&w=majority`)
  - A `DATABASE` key, with the name of the database to use
  
  Optionally, if **Birthbot** is compiled with the `guild` Cargo feature, it must also contain a `GUILD` key with a guild ID.

- ## Database

  **Birthbot** stores all its data on a MongoDB database as specified by the `.env` file.
  Separate collections are used for each guild, in which separate documents are used to store each user's birthday.

# Features

- ## Commands

  **Birthbot** recognises the following slash commands:
    - `birthday get` to retrieve a user's birthday
    - `birthday set` to add or update the command user's birthday
    - `birthday unset` to remove the command user's birthday
    - `birthday announce` to add or update the channel used for birthday announcements
    - `birthday unannounce` to remove the channel used for birthday announcements

- ## Announcements

  Every 24 hours beginning from its startup time, **Birthbot** checks its database for any ongoing birthdays, taking timezones into account.

  If any matches are found, they are announced in the channel as specified by `birthday announce`.
  If no announcement channel is set up, the birthdays are ignored.

# Errors

If and when an error occurs, **Birthbot** attempts to notify the command user of it.
Every error will also be printed to the standard error output.

Thanks to Rust's fantastic error handling, it is nearly impossible for **Birthbot** to crash due to an error.

# Privacy

**Birthbot** stores the bare minimum amount of data necessary to perform its tasks.
This includes:
- Guild IDs - *used as collection identifiers*
- User IDs - *used as document identifiers*
- Channel IDs - *used to announce birthdays*
- Birthdays - *used to check birthdays*