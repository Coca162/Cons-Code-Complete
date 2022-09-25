use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use std::ops::Add;
use std::time::{Duration, SystemTime};
use tracing::info;
use std::time::UNIX_EPOCH;
use dashmap::DashMap;
use std::collections::hash_map::RandomState;
use dashmap::DashSet;

use parse_duration::parse;

use crate::NoteContainer;

#[command]
pub async fn create(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {

    let topic = args.single::<String>().unwrap();
    
    let note = args.raw().skip(1).collect::<Vec<&str>>().join(" ");

    let data = ctx.data.read().await;

    if let Some(manager) = data.get::<NoteContainer>() {
        if let Some(user) = manager.get(&msg.author.id) {
            if let Some(topic) = user.get(&topic){
                topic.insert(note);
            }
            else {
                let mut dashset = DashSet::<String>::new();
    
                dashset.insert(note);
                user.insert(topic, dashset);
            }
        }
        else {
            let dashmap: DashMap<String, DashSet<String>> = DashMap::new();
            let mut dashset = DashSet::<String>::new();

            dashset.insert(note);
            dashmap.insert(topic, dashset);

            manager.insert(msg.author.id, dashmap);
        }
    } else {
        msg.reply(ctx, "There was a problem getting the focus list").await?;

        return Ok(());
    }

    msg.reply(ctx, "Created Note!").await?;


    Ok(())
}

#[command]
pub async fn list(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    
    let topic = args.single::<String>().unwrap();
    
    let data = ctx.data.read().await;

    if let Some(manager) = data.get::<NoteContainer>() {
        if let Some(user) = manager.get(&msg.author.id) {
            if let Some(topic) = user.get(&topic) {
                for note in topic.iter() {
                    msg.reply(ctx, note.to_string()).await?;
                }
            }
            else {
                msg.reply(ctx, "No notes were found").await?;
            }
        }
    } else {
        msg.reply(ctx, "There was a problem getting the focus list").await?;

        return Ok(());
    }

    Ok(())
}