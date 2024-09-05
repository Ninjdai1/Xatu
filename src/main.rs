#[allow(dead_code)]
mod github;

use std::{env, error::Error};

//use chrono::Utc;
use dotenvy::dotenv;
//use github::GithubData;
use secrecy::Secret;
use serenity::all::Client;
use tokio_cron_scheduler::{/*JobBuilder,*/ JobScheduler};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let sched = JobScheduler::new().await.expect("Error starting job scheduler");
    dotenv().expect(".env file not found");

    octocrab::initialise(octocrab::Octocrab::builder()
        .personal_token(Secret::new(env::var("GITHUB_TOKEN").expect("Expected a token in the environment")))
        .build().unwrap()
    );

    /*sched.add(
        JobBuilder::new()
            .with_timezone(Utc)
            .with_cron_job_type()
            .with_schedule("0 * * * * *")
            .unwrap()
            .with_run_async(Box::new(|_uuid, mut _l| {
                Box::pin(async move {
                    let mut github_data = GithubData::new();
                    github_data.fetch().await;

                    let output = github_data.render();
                    if let Ok(gist_id) = env::var("GIST_ID") {
                        octocrab::instance().gists()
                            .update(gist_id)
                            .description(format!("RHH Expansion Stats - {time}", time=chrono::offset::Utc::now()))
                            .file("rhhstats.md")
                            .with_content(output)
                            .send().await.expect("Failed updating gist");
                    }
                })
            })).build().expect("Error creating fetch job")
    ).await.expect("Error adding fetch job");*/

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = Default::default();

    let mut client =
        Client::builder(&token, intents).await.expect("Err creating client");

    if let Err(why) = sched.start().await {
        println!("Scheduler error: {why:?}");
    };

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    };
    Ok(())
}
