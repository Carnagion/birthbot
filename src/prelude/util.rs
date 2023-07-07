use std::fmt::Debug;

use log::error;

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

/// Extension trait for embed colours and titles commonly used by the bot.
pub trait CreateEmbedExt {
    /// Modifies an embed to have a border colour and title representing a successful operation.
    fn success(&mut self) -> &mut Self;

    /// Modifies an embed to have a border colour and title representing the lack of any significant change in data.
    fn unavailable(&mut self) -> &mut Self;

    /// Modifies an embed to have a border colour and title representing a failed operation.
    fn error(&mut self) -> &mut Self;
}

impl CreateEmbedExt for CreateEmbed {
    fn success(&mut self) -> &mut Self {
        self.title("Success").colour(POSITIVE)
    }

    fn unavailable(&mut self) -> &mut Self {
        self.title("Unavailable").colour(BLURPLE)
    }

    fn error(&mut self) -> &mut Self {
        self.title("Error").colour(DANGER)
    }
}

/// Wrapper for constructing command reply consisting of nothing but an embed.
pub async fn embed(
    context: &BotContext<'_>,
    ephemeral: bool,
    embed_builder: impl FnOnce(&mut CreateEmbed) -> &mut CreateEmbed,
) -> Result<()> {
    context
        .send(|response| response.ephemeral(ephemeral).embed(embed_builder))
        .await?;
    Ok(())
}

/// Extension trait for converting any [`Serialize`]-able type to BSON through a method.
pub trait SerializeExt: Serialize {
    /// Converts the value to [`Bson`].
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

/// Reports an error that occurred during the bot's running time.
///
/// This will attempt to display the error to the user if possible, using an embed for consistency with the rest of the bot's replies.
/// All errors and failures to report them are also logged.
pub async fn report_framework_error(error: BotFrameworkError<'_>) {
    let result = match error {
        BotFrameworkError::Command { error, ctx } => {
            error!("Command execution failed: {}", error);
            embed(&ctx, true, |embed| {
                embed
                    .error()
                    .description("The command failed to execute to completion.")
            })
            .await
        },
        BotFrameworkError::ArgumentParse { error, input, ctx } => {
            embed(&ctx, true, |embed| {
                embed
                    .error()
                    .description("One of the arguments provided is invalid.");
                if let Some(arg) = input {
                    embed.field(
                        "Argument",
                        format!(
                            "```\n{}\n```",
                            arg.replace("```", "\u{200B}`\u{200B}`\u{200B}`") // NOTE: Zero width spaces
                        ),
                        false,
                    );
                }
                embed.field("Details", error, true)
            })
            .await
        },
        BotFrameworkError::CommandStructureMismatch { description, ctx } => {
            error!("Command structure mismatch: {}", description);
            embed(&BotContext::Application(ctx), true, |embed| {
                embed
                    .error()
                    .description("The command is structured incorrectly.")
            })
            .await
        },
        BotFrameworkError::CommandCheckFailed { error, ctx } => {
            if let Some(error) = error {
                error!("Command verification error: {}", error);
            }
            embed(&ctx, true, |embed| {
                embed
                    .error()
                    .description("One of the command verification checks failed.")
            })
            .await
        },
        BotFrameworkError::CooldownHit {
            remaining_cooldown,
            ctx,
        } => {
            embed(&ctx, true, |embed| {
                embed
                    .error()
                    .description("A cooldown is still active.")
                    .field(
                        "Time remaining",
                        format!("`{}` second(s)", remaining_cooldown.as_secs()),
                        true,
                    )
            })
            .await
        },
        BotFrameworkError::MissingBotPermissions {
            missing_permissions,
            ctx,
        } => {
            embed(&ctx, true, |embed| {
                embed
                    .error()
                    .description("More permissions are required to execute your command.")
                    .field(
                        "Missing permissions",
                        format!("```\n{}\n```", missing_permissions),
                        true,
                    )
            })
            .await
        },
        BotFrameworkError::MissingUserPermissions {
            missing_permissions,
            ctx,
        } => {
            embed(&ctx, true, |embed| {
                embed
                    .error()
                    .description("You require more permissions to use that command.");
                if let Some(missing_permissions) = missing_permissions {
                    embed.field(
                        "Missing permissions",
                        format!("```\n{}\n```", missing_permissions),
                        true,
                    );
                }
                embed
            })
            .await
        },
        BotFrameworkError::NotAnOwner { ctx } => {
            embed(&ctx, true, |embed| {
                embed
                    .error()
                    .description("Only the guild owner can use that command.")
            })
            .await
        },
        BotFrameworkError::GuildOnly { ctx } => {
            embed(&ctx, true, |embed| {
                embed
                    .error()
                    .description("That command can only be used in guilds.")
            })
            .await
        },
        BotFrameworkError::DmOnly { ctx } => {
            embed(&ctx, true, |embed| {
                embed
                    .error()
                    .description("That command can only be used in direct messages.")
            })
            .await
        },
        BotFrameworkError::NsfwOnly { ctx } => {
            embed(&ctx, true, |embed| {
                embed
                    .error()
                    .description("That command can only be used in NSFW channels.")
            })
            .await
        },
        BotFrameworkError::Setup { error, .. } => {
            error!("User data setup failed: {}", error);
            Ok(())
        },
        BotFrameworkError::EventHandler { error, event, .. } => {
            error!("Event handler for {} failed: {}", event.name(), error);
            Ok(())
        },
        BotFrameworkError::DynamicPrefix { error, msg, .. } => {
            error!(r#"Dynamic prefix failed on "{}": {}"#, msg.content, error);
            Ok(())
        },
        BotFrameworkError::UnknownCommand {
            prefix,
            msg_content,
            ..
        } => {
            error!(
                r#"Unrecognised command "{}" for prefix "{}""#,
                msg_content, prefix
            );
            Ok(())
        },
        BotFrameworkError::UnknownInteraction { .. } => {
            error!("Unknown interaction");
            Ok(())
        },
        _ => {
            error!("Unknown error");
            Ok(())
        },
    };

    if let Err(error) = result {
        error!("Error reporting failed: {}", error);
    }
}
