create table if not exists announcements (
    guild_id integer not null,
    channel_id integer,
    unique(guild_id)
);