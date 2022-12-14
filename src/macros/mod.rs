macro_rules! command_response {
    ($command:expr, $context:expr, $data:expr) => {
        $command.create_interaction_response(&$context.http, |response| response
                .kind(serenity::model::prelude::interaction::InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data($data))
            .await
            .map_err(crate::errors::BotError::SerenityError)
    }
}

macro_rules! command_error {
    ($description:expr, $command:expr, $context:expr) => {
        command_response!($command, $context, |data| data
                .ephemeral(true)
                .embed(|embed| embed
                    .title("Error")
                    .description($description)
                    .colour(serenity::utils::Colour::from_rgb(237, 66, 69))))
    };
}

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

macro_rules! require_command_simple_option {
    ($option:expr, $kind:ident, $name:expr) => {
        match $option.map_or_else(|| Err(crate::errors::BotError::CommandError(format!(r#"The option "{}" is expected."#, $name))), |option| resolve_command_option!(option, $kind, $name))? {
            serenity::model::application::interaction::application_command::CommandDataOptionValue::$kind(value) => Ok(value),
            _ => Err(crate::errors::BotError::CommandError(format!(r#"The resolved value for the option "{}" is invalid."#, $name))),
        }
    };
    ($option:expr, $kind:ident, $name:expr, $default:expr) => {
        match $option.map_or_else(|| Ok(&serenity::model::application::interaction::application_command::CommandDataOptionValue::$kind($default)), |option| resolve_command_option!(option, $kind, $name))? {
            serenity::model::application::interaction::application_command::CommandDataOptionValue::$kind(value) => Ok(value),
            _ => Err(crate::errors::BotError::CommandError(format!(r#"The resolved value for the option "{}" is invalid."#, $name))),
        }
    };
}

macro_rules! require_command_user_option {
    ($option:expr, $name:expr, $default:expr) => {
        match $option.map(|option_user| resolve_command_option!(option_user, User, $name)) {
            None => $default,
            Some(result) => match result? {
                serenity::model::application::interaction::application_command::CommandDataOptionValue::User(user, _) => Ok(user),
                _ => Err(crate::errors::BotError::CommandError(format!(r#"The resolved value for the option "{}" is invalid."#, $name))),
            }?,
        }
    };
}

macro_rules! bson_birthday {
    () => {
        mongodb::bson::doc! {
            "user": {
                "$exists": true,
                "$type": "long",
            },
            "birth.day": {
                "$exists": true,
                "$type": "int",
            },
            "birth.month": {
                "$exists": true,
                "$type": "int",
            },
            "birth.year": {
                "$exists": true,
                "$type": "int",
            },
            "birth.offset": {
                "$exists": true,
                "$type": "int",
            },
        }
    };
    ($id:expr) => {
        mongodb::bson::doc! {
            "user": $id,
            "birth.day": {
                "$exists": true,
                "$type": "int",
            },
            "birth.month": {
                "$exists": true,
                "$type": "int",
            },
            "birth.year": {
                "$exists": true,
                "$type": "int",
            },
            "birth.offset": {
                "$exists": true,
                "$type": "int",
            },
        }
    };
}