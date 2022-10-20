//! Generates and executes the asynchronous task for checking birthdays.

use std::time::Duration;

use chrono::Datelike;
use chrono::Utc;

use mongodb::Collection;
use mongodb::bson;
use mongodb::bson::Document;

use serenity::client::Context;
use serenity::model::id::ChannelId;
use serenity::utils::Colour;

use tokio;
use tokio::time;

use crate::errors::BotError;

/// Spawns an asynchronous task to check for birthdays every day.
pub fn create_birthday_scheduler(context: &Context) {
    let cloned = context.clone();
    tokio::spawn(async move {
        if let Err(error) = loop_checks(&cloned).await {
            eprintln!("{:?}", error);
        }
    });
}

async fn loop_checks(context: &Context) -> Result<(), BotError> {
    loop {
        check_birthdays(context).await?;
        time::sleep(Duration::from_secs(86400)).await;
    }
}

async fn check_birthdays(context: &Context) -> Result<(), BotError> {
    // Connect to database
    let database = super::connect_mongodb().await?;
    // Retrieve all collections in database
    let names = database
        .list_collection_names(None)
        .await?;
    for name in names {
        let query = bson_birthday!();
        // Retrieve all documents in collection
        let collection = database.collection::<Document>(name.as_str());
        let mut documents = collection
            .find(query, None)
            .await?;
        while documents.advance().await? {
            // Check if current server day is user's birthday
            let document = documents.deserialize_current()?;
            let user = document.get_i64("user")?;
            let birth_date = super::get_birthday(&document)?
                .with_timezone(&Utc);
            let server_date = Utc::now();
            // If birthday, announce in channel
            if birth_date.day() == server_date.day() && birth_date.month() == server_date.month() {
                let age = server_date.year() - document
                    .get_document("birth")?
                    .get_i32("year")?;
                announce_birthday(user, age, &collection, context).await?;
            }
        }
    }
    Ok(())
}

async fn announce_birthday(user: i64, age: i32, collection: &Collection<Document>, context: &Context) -> Result<(), BotError> {
    // Retrieve channel ID from collection
    let query = bson::doc! {
        "config.channel": {
            "$exists": true,
            "$type": "long",
        },
    };
    let config = collection
        .find_one(query, None)
        .await?;
    // If channel ID does not exist, no announcement is made
    if let Some(config) = config {
        ChannelId(config
            .get_document("config")?
            .get_i64("channel")? as u64)
            .send_message(&context.http, |message| message
                .embed(|embed| embed
                    .title("Birthday")
                    .description(format!("It's <@{}>'s birthday!", user))
                    .field("Age", age, true)
                    .colour(Colour::from_rgb(235, 69, 158))))
            .await?;
    }
    Ok(())
}