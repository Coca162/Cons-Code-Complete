//! Requires the 'framework' feature flag be enabled in your project's
//! `Cargo.toml`.
//!
//! This can be enabled by specifying the feature in the dependency section:
//!
//! ```toml
//! [dependencies.serenity]
//! git = "https://github.com/serenity-rs/serenity.git"
//! features = ["framework", "standard_framework"]
//! ```
mod commands;

use std::collections::HashSet;
use std::env;
use std::str::FromStr;
use std::sync::Arc;
use dashmap::DashMap;

use dashmap::DashSet;
use serenity::async_trait;
use serenity::client::bridge::gateway::ShardManager;
use serenity::framework::standard::help_commands::SuggestedCommandName;
use serenity::framework::standard::macros::group;
use serenity::framework::StandardFramework;
use serenity::http::Http;
use serenity::model::event::ResumedEvent;
use serenity::model::gateway::Ready;
use serenity::model::prelude::Message;
use serenity::prelude::*;
use tracing::{error, info};
use std::time::{Duration, SystemTime};
use serenity::model::channel;
use serenity::model::prelude::ChannelId;

use crate::commands::focus::*;
use crate::commands::meta::*;
use crate::commands::owner::*;
use crate::commands::notes::*;

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

pub struct FocusedUsersContainer;

impl TypeMapKey for FocusedUsersContainer {
    type Value = DashMap<serenity::model::id::UserId, (SystemTime, Duration)>;
}

pub struct NoteContainer;

impl TypeMapKey for NoteContainer {
    type Value = DashMap<serenity::model::id::UserId, DashMap<String, DashSet<String>>>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }

    async fn message(&self, ctx: Context, msg: Message) {
        let data = ctx.data.read().await;

        if let Some(manager) = data.get::<FocusedUsersContainer>() {
            if manager.into_iter().any(|focus| {
                msg.content.contains(&focus.key().as_u64().to_string())
            })
            {
                msg.reply(&ctx, "DO NOT PING FOCUSED USERS").await.unwrap();
                let guild = msg.guild(&ctx).unwrap();
                guild.kick(&ctx, &msg.author).await.unwrap();
            }
        } else {
            msg.reply(&ctx, "There was a problem getting the focus list").await.unwrap();
        }

        if let Some(manager) = data.get::<FocusedUsersContainer>() {
            let option = manager.get(&msg.author.id);
            if let Some(user) = option {
                if user.0.elapsed().unwrap() < user.1 {
                    let guild = msg.guild(&ctx).unwrap();
                    guild.kick(&ctx, &msg.author).await.unwrap();
                                    
                    let channel = ChannelId(1023646930980585542);

                    channel.send_message(&ctx.http, |m| {
                        m
                        .content(format!("{} was kicked for speaking when they were meant to focus", msg.author.name))
                        .tts(true)
                    }).await.unwrap();

                    info!("Banned");
                }
            }
        } else {
            msg.reply(&ctx, "There was a problem getting the focus list").await.unwrap();
        }
    }

}

#[group]
#[commands(focus, ping, quit, listall)]
struct General;

#[group]
#[prefix = "note"]
#[commands(create, list)]
struct Notes;

#[tokio::main]
async fn main() {
    // This will load the environment variables located at `./.env`, relative to
    // the CWD. See `./.env.example` for an example on how to structure this.
    dotenv::dotenv().expect("Failed to load .env file");

    // Initialize the logger to use environment variables.
    //
    // In this case, a good default is setting the environment variable
    // `RUST_LOG` to `debug`.
    tracing_subscriber::fmt::init();

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let http = Http::new(&token);

    // We will fetch your bot's owners and id
    let (owners, _bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        },
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    // Create the framework
    let framework =
        StandardFramework::new().configure(|c| c.owners(owners).prefix("~")).group(&GENERAL_GROUP).group(&NOTES_GROUP);

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_BANS | GatewayIntents::GUILDS;
    let mut client = Client::builder(&token, intents)
        .framework(framework)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
        data.insert::<FocusedUsersContainer>(DashMap::new());
        data.insert::<NoteContainer>(DashMap::new());
    }

    let shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.expect("Could not register ctrl+c handler");
        shard_manager.lock().await.shutdown_all().await;
    });

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}

