//! Models for guild data and configuration.

use mongodm::prelude::*;

use poise::serenity_prelude as serenity;

use serde::{Deserialize, Serialize};

use serenity::*;

/// Configuration-related data of a guild.
#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct GuildData {
    /// The guild's ID.
    pub guild_id: GuildId,
    /// The ID of the channel where birthdays are announced.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub birthday_channel_id: Option<ChannelId>,
}

impl Model for GuildData {
    type CollConf = GuildDataCollection;
}

/// Collection name and indexes for [`GuildData`].
#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct GuildDataCollection;

impl CollectionConfig for GuildDataCollection {
    fn collection_name() -> &'static str {
        "guild-data"
    }

    fn indexes() -> Indexes {
        Indexes::new()
            .with(Index::new(field!(guild_id in GuildData)).with_option(IndexOption::Unique))
    }
}
