use std::fmt::Debug;

use mongodm::prelude::*;

use poise::serenity_prelude::{
    colours::{
        branding::BLURPLE,
        css::{DANGER, POSITIVE},
    },
    *,
};

use serde::Serialize;

use snafu::prelude::*;

use crate::prelude::*;

pub trait CreateEmbedExt {
    fn success(&mut self) -> &mut Self;

    fn unchanged(&mut self) -> &mut Self;

    fn error(&mut self) -> &mut Self;
}

impl CreateEmbedExt for CreateEmbed {
    fn success(&mut self) -> &mut Self {
        self.title("Success").colour(POSITIVE)
    }

    fn unchanged(&mut self) -> &mut Self {
        self.title("Unchanged").colour(BLURPLE)
    }

    fn error(&mut self) -> &mut Self {
        self.title("Error").colour(DANGER)
    }
}

pub async fn embed(
    context: &BotContext<'_>,
    ephemeral: bool,
    embed_builder: impl FnOnce(&mut CreateEmbed) -> &mut CreateEmbed,
) -> BotResult<()> {
    context
        .send(|response| response.ephemeral(ephemeral).embed(embed_builder))
        .await?;
    Ok(())
}

pub trait SerializeExt: Serialize {
    fn to_bson(&self) -> BotResult<Bson>;
}

impl<T> SerializeExt for T
where
    T: Serialize + Debug,
{
    fn to_bson(&self) -> BotResult<Bson> {
        to_bson(self).with_context(|_| BsonSerSnafu {
            debug: format!("{:?}", self),
        })
    }
}
