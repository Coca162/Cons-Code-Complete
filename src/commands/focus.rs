use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use std::ops::Add;
use std::time::{Duration, SystemTime};
use tracing::info;
use std::time::UNIX_EPOCH;

use parse_duration::parse;

use crate::FocusedUsersContainer;


#[command]
pub async fn focus(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let whole_date = args.raw().skip(1).collect::<Vec<&str>>().join(" ");
    
    let duration = parse(&whole_date);

    if duration.is_err() {
        msg.reply(ctx, "This is not a duration!").await?;

        return Ok(());
    }

    let now = SystemTime::now();
    let duration = duration.unwrap();

    let data = ctx.data.read().await;

    if let Some(manager) = data.get::<FocusedUsersContainer>() {
        manager.insert(msg.author.id, (now, duration));
    } else {
        msg.reply(ctx, "There was a problem getting the focus list").await?;

        return Ok(());
    }

    let work_task = args.single::<String>().unwrap();
    
    let channel = ChannelId(1023646930980585542);

    let unit_timestamp = now.add(duration).duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs();

    channel.send_message(&ctx.http, |m| {
        m
        .content(format!("{} is focusing on {} and will finish at <t:{}:R>", msg.author.name, work_task, unit_timestamp))
        .tts(true)
    }).await.unwrap();

    msg.channel_id.say(&ctx.http, "Focusing!").await?;

    Ok(())
}