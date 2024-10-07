#[allow(dead_code)]
mod github;
mod utils;

use std::{env, sync::{atomic::{AtomicBool, Ordering}, Arc}, time::Duration};
use dotenvy::dotenv;
use github::GithubData;
use serenity::{all::{ChannelId, Client, Context, EventHandler, GuildId, Message, Ready}, async_trait, Error};

struct Handler {
    is_loop_running: AtomicBool,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content.starts_with("!ping") {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                eprintln!("Error sending message: {why:?}");
            }
        }
    }

    async fn ready(&self, _ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }

    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
        println!("Cache built successfully!");
        let ctx = Arc::new(ctx);

        if !self.is_loop_running.load(Ordering::Relaxed) {
            let ctx1 = Arc::clone(&ctx);
            tokio::spawn(async move {
                loop {
                    println!("Starting loading GH data");
                    let mut github_data = GithubData::new();
                    github_data.fetch().await;

                    let output = github_data.render();
                    println!("Github data loaded");

                    let channel_id = ChannelId::new(875622508026544148);
                    if let Err(why) = channel_id.say(&ctx1.http, &output).await {
                        println!("Error sending message in channel: {:?}", why);
                    }
                    tokio::time::sleep(Duration::from_secs(400)).await;
                }
            });
            self.is_loop_running.swap(true, Ordering::Relaxed);
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().expect(".env file not found");

    octocrab::initialise(octocrab::Octocrab::builder()
        .personal_token(env::var("GITHUB_TOKEN").expect("Expected a token in the environment"))
        .build().unwrap()
    );

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = Default::default();

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler {
            is_loop_running: AtomicBool::new(false),
        })
        .await
        .expect("Err creating client");

    client.start().await?;
    Ok(())
}
