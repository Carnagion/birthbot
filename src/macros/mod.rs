#[macro_export]
macro_rules! command_response {
    ($message:expr, $command:expr, $context:expr) => {
        {
            use serenity::model::prelude::interaction::InteractionResponseType;

            let result = $command.create_interaction_response(&($context).http, |response| response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|data| data
                        .content($message)
                        .ephemeral(true)))
                .await;
            if let Err(error) = result {
                println!("Error responding to interaction: {}", error);
            }
        }
    }
}

pub use command_response;

#[macro_export]
macro_rules! resolve_command_option {
    ($option:expr, $name:expr, $type:ident, $command:expr, $context:expr) => {
        {
            use serenity::model::application::command::CommandOptionType;

            use crate::macros;

            match &($option).kind {
                CommandOptionType::$type => {
                    match &($option).resolved {
                        None => {
                            macros::command_response!(format!(r#"The parameter "{}" is unresolved."#, $name), $command, $context);
                            None
                        },
                        Some(resolved) => Some(resolved),
                    }
                },
                _ => {
                    macros::command_response!(format!(r#"The type for the parameter "{}" is invalid."#, $name), $command, $context);
                    None
                },
            }
        }
    };
}

pub use resolve_command_option;

#[macro_export]
macro_rules! require_command_option {
    ($option:expr, $name:expr, $type:ident, $command:expr, $context:expr) => {
        {
            use serenity::model::application::interaction::application_command::CommandDataOptionValue;

            use crate::macros;

            let mut required = None;
            match $option {
                None => macros::command_response!(format!(r#"The parameter "{}" is required."#, $name), $command, $context),
                Some(option) => {
                    if let Some(resolved) = macros::resolve_command_option!(option, $name, $type, $command, $context) {
                        match resolved {
                            CommandDataOptionValue::$type(value) => required = Some(value),
                            _ => macros::command_response!(format!(r#"The resolved value for the parameter "{}" is invalid."#, $name), $command, $context),
                        }
                    }
                }
            }
            required
        }
    }
}

pub use require_command_option;

#[macro_export]
macro_rules! require_command_user_or_default {
    ($option:expr, $command:expr, $context:expr) => {
        {
            use crate::macros;

            match $option {
                None => Some(&($command).user),
                Some(option) => {
                    match macros::resolve_command_option!(option, "user", User, $command, $context) {
                        None => None,
                        Some(resolved) => {
                            match resolved {
                                CommandDataOptionValue::User(user, _) => Some(user),
                                _ => {
                                    macros::command_response!(r#"The resolved value for the parameter "user" is invalid."#, $command, $context);
                                    None
                                },
                            }
                        }
                    }
                }
            }
        }
    }
}

pub use require_command_user_or_default;