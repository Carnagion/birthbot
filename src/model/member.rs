use mongodm::{prelude::*, Indexes};

use poise::serenity_prelude::*;

use serde::{Deserialize, Serialize};

use crate::prelude::*;

/// Birthday-related data of a member.
#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct MemberData {
    /// The member's user ID.
    ///
    /// Together with the guild ID, this uniquely identifies a member.
    pub user_id: UserId,
    /// The member's guild ID.
    ///
    /// Together with the user ID, this uniquely identifies a member.
    pub guild_id: GuildId,
    /// The member's birthday.
    pub birthday: Birthday,
}

impl Model for MemberData {
    type CollConf = MemberDataCollection;
}

/// Collection name and indexes for [`MemberData`].
#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MemberDataCollection;

impl CollectionConfig for MemberDataCollection {
    fn collection_name() -> &'static str {
        "member-data"
    }

    fn indexes() -> Indexes {
        Indexes::new().with(
            Index::new(field!(user_id in MemberData))
                .with_key(field!(guild_id in MemberData))
                .with_option(IndexOption::Unique),
        )
    }
}
