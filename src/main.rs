use lazy_static::lazy_static;
use rand::seq::SliceRandom;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use toml;

use reqwest::header::{HeaderMap, HeaderValue, COOKIE};
use serde_json::Value;
use serenity::async_trait;
use serenity::builder::{CreateEmbed, CreateMessage};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::id::{ChannelId, GuildId};
use serenity::prelude::*;

lazy_static! {
    static ref DISCORD_TOKEN: Mutex<String> =
        Mutex::new(env::var("DISCORD_TOKEN").unwrap_or_else(|_| {
            secret_from_toml("DISCORD_TOKEN")
                .expect("Expected a DISCORD_TOKEN token in the environment or Secrets.toml")
        }));
    static ref OVERLEAF_SESSION_KEY: Mutex<String> =
        Mutex::new(env::var("OVERLEAF_SESSION_KEY").unwrap_or_else(|_| {
            secret_from_toml("OVERLEAF_SESSION_KEY")
                .expect("Expected a OVERLEAF_SESSION_KEY in the environment or Secrets.toml")
        }));
    static ref DISCORD_CHANNEL: Mutex<u64> = Mutex::new(
        env::var("DISCORD_CHANNEL")
            .unwrap_or_else(|_| {
                secret_from_toml("DISCORD_CHANNEL")
                    .expect("Expected a DISCORD_CHANNEL token in the environment or Secrets.toml")
            })
            .parse()
            .expect("Failed to parse DISCORD_CHANNEL as u64")
    );
    static ref OVERLEAF_DOC_ID: Mutex<String> =
        Mutex::new(env::var("OVERLEAF_DOC_ID").unwrap_or_else(|_| {
            secret_from_toml("OVERLEAF_DOC_ID")
                .expect("Expected a OVERLEAF_DOC_ID in the environment or Secrets.toml")
        }));
}

const MIN_WORDS: u32 = 2500;

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

    // We use the cache_ready event just in case some cache operation is required in whatever use
    // case you have for this.
    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
        println!("Cache built successfully!");

        // It's safe to clone Context, but Arc is cheaper for this use case.
        // Untested claim, just theoretically. :P
        let ctx = Arc::new(ctx);

        let mut last_remembered_word_count = 0;

        if !self.is_loop_running.load(Ordering::Relaxed) {
            tokio::spawn(async move {
                loop {
                    if let Ok(word_count) = get_words().await {
                        if (word_count as f32) > (last_remembered_word_count as f32 * 1.01) {
                            last_remembered_word_count = word_count;
                            send_speechless(&ctx, word_count).await;
                            send_embed(&ctx, word_count).await;
                            send_motivation(&ctx).await;
                        }
                    };
                    tokio::time::sleep(Duration::from_secs(28800)).await;
                }
            });

            // Now that the loop is running, we set the bool to true
            self.is_loop_running.swap(true, Ordering::Relaxed);
        }
    }
}

async fn send_speechless(ctx: &Context, word_count: u32) {
    let percentage = (word_count * 100) / MIN_WORDS;
    if percentage > 100 {
        // If we're reached 100%, there is no need for that extra motivation
        return;
    }
    let speechless_sentences = [
        format!("Wow, check it, we're at **{}%** !!", percentage),
        format!("We're on fire! Just reached **{}%**!", percentage),
        format!(
            "Making great progress with the **{}%**, Amazing!",
            percentage
        ),
        format!("Guys, **{}**% now, impressive nice !!", percentage),
        format!("Amazing work, team! We've reached **{}%** !", percentage),
        format!(
            "Keep it up, everyone! We're already at **{}%** !",
            percentage
        ),
        format!("Fantastic job, we're at **{}%** !", percentage),
        format!(
            "Keep up the good work, incredible, look at that beautiful **{}%** !",
            percentage
        ),
        format!(
            "Absolutely beautiful guys, **{}%**, just so good!",
            percentage
        ),
        format!("**{}%**, Lessgo!!!", percentage),
    ];

    let discord_channel: u64 = { *DISCORD_CHANNEL.lock().await };

    let sentence = speechless_sentences
        .choose(&mut rand::thread_rng())
        .unwrap_or(&speechless_sentences[0]);

    let builder = CreateMessage::new().content(sentence);
    let message = ChannelId::new(discord_channel)
        .send_message(&ctx, builder)
        .await;
    if let Err(why) = message {
        eprintln!("Error sending message: {:?}", why);
    };
}

async fn send_motivation(ctx: &Context) {
    let discord_channel: u64 = { *DISCORD_CHANNEL.lock().await };

    let motivational_sentences = [
        "Believe you can and you're halfway there",
        "Don't watch the clock; do what it does. Keep going",
        "The secret of getting ahead is getting started",
        "The future depends on what you do today",
        "Believe in yourself and all that you are. Know that there is something inside you that is greater than any obstacle",
        "The only way to achieve the impossible is to believe it is possible",
        "Success is not the key to happiness. Happiness is the key to success. If you love what you are doing, you will be successful",
        "The only limit to our realization of tomorrow will be our doubts of today",
        "The only place where success comes before work is in the dictionary",
        "Together, we can conquer any challenge",
        "Every step we take together brings us one step closer to our goal",
        "We are a team. We win together, we lose together, we work hard together",
        "The strength of the team is each individual member. The strength of each member is the team",
        "Alone we can do so little; together we can do so much",
        "Great things in business are never done by one person; they're done by a team of people",
        "Remember, we're all in this together. Every effort counts",
        "We got this! Let's keep pushing forward",
        "Teamwork divides the task and multiplies the success",
        "Together, we can conquer any challenge. Let's hit that 2500 word count!",
        "Every word we write together brings us one step closer to our 2500-word goal",
        "We are a team. We write together, we edit together, we reach our word count together",
        "The strength of our report is each individual word. The strength of each word is our team",
        "Alone we can write so little; together we can write so much. Let's reach that 2500 words!",
        "Great reports are never written by one person; they're written by a team of people",
        "Remember, we're all in this together. Every word counts towards our 2500-word goal",
        "We got this! Let's keep pushing forward until we reach 2500 words",
        "Teamwork divides the task and multiplies the success. Let's multiply our success by reaching 2500 words",
        "Let's continue to work hard and support each other. 2500 words, here we come!",
        "The harder you work for something, the greater you'll feel when you achieve it"
    ];

    let sentence = motivational_sentences
        .choose(&mut rand::thread_rng())
        .unwrap_or(&motivational_sentences[0]);

    let builder = CreateMessage::new().content(sentence.to_string());
    let message = ChannelId::new(discord_channel)
        .send_message(&ctx, builder)
        .await;
    if let Err(why) = message {
        eprintln!("Error sending message: {:?}", why);
    };
}

async fn send_embed(ctx: &Context, word_count: u32) {
    let discord_channel: u64 = { *DISCORD_CHANNEL.lock().await };
    // We can use ChannelId directly to send a message to a specific channel; in this case, the
    // message would be sent to the #testing channel on the discord server.
    let percentage = (word_count * 100) / MIN_WORDS;
    let embed = CreateEmbed::new().title("Overleaf word count").field(
        format!("{}/{} ({:.2}%)", word_count, MIN_WORDS, percentage),
        "",
        false,
    );
    let builder = CreateMessage::new().embed(embed);
    let message = ChannelId::new(discord_channel)
        .send_message(&ctx, builder)
        .await;
    if let Err(why) = message {
        eprintln!("Error sending message: {why:?}");
    };
}

async fn get_words() -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
    let overleaf_doc_id = { (*OVERLEAF_DOC_ID.lock().await).clone() };
    let overleaf_session = { (*OVERLEAF_SESSION_KEY.lock().await).clone() };
    let url = format!(
        "https://www.overleaf.com/project/{}/wordcount",
        overleaf_doc_id
    );

    let mut headers = HeaderMap::new();
    headers.insert(
        COOKIE,
        HeaderValue::from_str(&format!("overleaf_session2={}", overleaf_session))?,
    );

    let client = reqwest::Client::new();
    let res = client.get(url).headers(headers).send().await?;
    let body: Value = res.json().await?;

    let word_count = body["texcount"]["textWords"]
        .as_u64()
        .ok_or("Failed to parse word count")? as u32;

    Ok(word_count)
}

fn secret_from_toml(key: &str) -> Result<String, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string("Secrets.toml")?;
    let secrets: HashMap<String, String> = toml::from_str(&contents)?;
    match secrets.get(key) {
        Some(value) => Ok(value.clone()),
        None => Err(format!("Key {} not found in Secrets.toml", key).into()),
    }
}

#[tokio::main]
async fn main() {
    let token = { (*DISCORD_TOKEN.lock().await).clone() };

    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::GUILDS;
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler {
            is_loop_running: AtomicBool::new(false),
        })
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        eprintln!("Client error: {why:?}");
    }
}
