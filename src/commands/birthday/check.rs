//! Generates and executes the `cron` task for checking birthdays.

use chrono::Datelike;
use chrono::Utc;

use cronjob::CronJob;

use mongodb::bson;
use mongodb::bson::Document;

use tokio;

use crate::errors::BotError;

/// Sets up the `cron` job that checks for birthdays coinciding with the current day.
pub fn setup_birthday_cron() {
    let mut cron = CronJob::new("birthday check", execute_birthday_cron);
    cron/*.seconds("5,10,15,20,25,30,35,40,45,50,55,60")
        .minutes("0")
        .hours("0")*/
        .day_of_month("*")
        .offset(0);
    CronJob::start_job_threaded(cron);
}

#[tokio::main] // Transforms the asynchronous function into a synchronous one, allowing it to be used with cron jobs
async fn execute_birthday_cron(_: &str) {
    if let Err(error) = handle_birthday_check_subcommand().await {
        println!("{:?}", error);
    }
}

async fn handle_birthday_check_subcommand() -> Result<(), BotError> {
    // Connect to database
    let database = super::connect_mongodb().await?;
    // Retrieve all collections in database
    let names = database
        .list_collection_names(None)
        .await?;
    for name in names {
        let query = bson::doc! {
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
        };
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
            if birth_date.day() == server_date.day() && birth_date.month() == server_date.month() {
                println!("{} ({}) - {}", user, birth_date, server_date);
            }
        }
    }
    Ok(())
}