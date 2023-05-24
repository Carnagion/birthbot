use mongodm::{prelude::*, Indexes};

use poise::serenity_prelude::*;

use serde::{Deserialize, Serialize};

use crate::prelude::*;

#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct MemberData {
    pub user_id: UserId,
    pub guild_id: GuildId,
    pub birthday: Birthday,
}

impl Model for MemberData {
    type CollConf = MemberDataCollection;
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MemberDataCollection;

impl CollectionConfig for MemberDataCollection {
    fn collection_name() -> &'static str {
        "member_data"
    }

    fn indexes() -> Indexes {
        Indexes::new().with(
            Index::new(field!(user_id in MemberData))
                .with_key(field!(guild_id in MemberData))
                .with_option(IndexOption::Unique),
        )
    }
}
