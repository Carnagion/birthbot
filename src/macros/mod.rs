#[macro_export]
macro_rules! command_response {
    ($message:expr, $command:expr, $context:expr, $ephemeral:expr) => {
        $command.create_interaction_response(&$context.http, |response| response
                .kind(serenity::model::prelude::interaction::InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|data| data
                    .content($message)
                    .ephemeral($ephemeral)))
            .await
    };
}

pub use command_response;

#[macro_export]
macro_rules! resolve_command_option {
    ($option:expr, $kind:ident, $name:expr) => {
        match &$option.kind {
            serenity::model::application::command::CommandOptionType::$kind => match &$option.resolved {
                None => Err(crate::errors::BotError::CommandError(format!(r#"The option "{}" is unresolved."#, $name))),
                Some(resolved) => Ok(resolved),
            },
            _ => Err(crate::errors::BotError::CommandError(format!(r#"The option "{}" has an invalid type."#, $name))),
        }
    };
}

pub use resolve_command_option;

#[macro_export]
macro_rules! require_command_int_option {
    ($option:expr, $name:expr) => {
        match $option.map_or_else(|| Err(crate::errors::BotError::CommandError(format!(r#"The option "{}" is expected."#, $name))), |option_int| crate::macros::resolve_command_option!(option_int, Integer, $name))? {
            serenity::model::application::interaction::application_command::CommandDataOptionValue::Integer(int) => Ok(int),
            _ => Err(crate::errors::BotError::CommandError(format!(r#"The resolved value for the parameter "{}" is invalid."#, $name))),
        }
    };
}

pub use require_command_int_option;

#[macro_export]
macro_rules! require_command_user_option {
    ($option:expr, $name:expr, $default:expr) => {
        match $option.map(|option_user| crate::macros::resolve_command_option!(option_user, User, $name)) {
            None => $default,
            Some(result) => match result? {
                serenity::model::application::interaction::application_command::CommandDataOptionValue::User(user, _) => Ok(user),
                _ => Err(crate::errors::BotError::CommandError(format!(r#"The resolved value for the parameter "{}" is invalid."#, $name))),
            }?,
        }
    };
}

pub use require_command_user_option;