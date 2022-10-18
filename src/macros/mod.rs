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
            .map_err(|error| println!("{:?}", error))
            .map_or((), |_| ())
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

macro_rules! require_command_int_option {
    ($option:expr, $name:expr) => {
        match $option.map_or_else(|| Err(crate::errors::BotError::CommandError(format!(r#"The option "{}" is expected."#, $name))), |option_int| resolve_command_option!(option_int, Integer, $name))? {
            serenity::model::application::interaction::application_command::CommandDataOptionValue::Integer(int) => Ok(int),
            _ => Err(crate::errors::BotError::CommandError(format!(r#"The resolved value for the parameter "{}" is invalid."#, $name))),
        }
    };
}

macro_rules! require_command_user_option {
    ($option:expr, $name:expr, $default:expr) => {
        match $option.map(|option_user| resolve_command_option!(option_user, User, $name)) {
            None => $default,
            Some(result) => match result? {
                serenity::model::application::interaction::application_command::CommandDataOptionValue::User(user, _) => Ok(user),
                _ => Err(crate::errors::BotError::CommandError(format!(r#"The resolved value for the parameter "{}" is invalid."#, $name))),
            }?,
        }
    };
}

macro_rules! bson_birthday {
    ($id:expr) => {
        mongodb::bson::doc! {
            format!("{}.birth.day", $id): {
                "$exists": true,
                "$type": "int",
            },
            format!("{}.birth.month", $id): {
                "$exists": true,
                "$type": "int",
            },
            format!("{}.birth.year", $id): {
                "$exists": true,
                "$type": "int",
            },
            format!("{}.birth.offset", $id): {
                "$exists": true,
                "$type": "int",
            },
        }
    };
}