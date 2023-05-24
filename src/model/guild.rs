use mongodm::{prelude::*, Indexes};

use poise::serenity_prelude::*;

use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct GuildData {
    pub guild_id: GuildId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub birthday_channel_id: Option<ChannelId>,
}

impl Model for GuildData {
    type CollConf = GuildDataCollection;
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct GuildDataCollection;

impl CollectionConfig for GuildDataCollection {
    fn collection_name() -> &'static str {
        "guild_data"
    }

    fn indexes() -> Indexes {
        Indexes::new()
            .with(Index::new(field!(guild_id in GuildData)).with_option(IndexOption::Unique))
    }
}
