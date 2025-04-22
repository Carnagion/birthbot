create table if not exists birthdays (
    user_id integer not null,
    guild_id integer not null,
    birthday text not null,
    unique(user_Id, guild_id)
);