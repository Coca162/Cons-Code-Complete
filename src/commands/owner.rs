use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandResult;
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::ShardManagerContainer;
use crate::FocusedUsersContainer;

#[command]
#[owners_only]
async fn quit(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;

    if let Some(manager) = data.get::<ShardManagerContainer>() {
        msg.reply(ctx, "Shutting down!").await?;
        manager.lock().await.shutdown_all().await;
    } else {
        msg.reply(ctx, "There was a problem getting the shard manager").await?;

        return Ok(());
    }

    Ok(())
}

#[command]
#[owners_only]
async fn listall(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;

    if let Some(manager) = data.get::<FocusedUsersContainer>() {
        for focus in manager.into_iter() {
            msg.reply(ctx, format!("{:?}, {:?} and {:?}", focus.key(), focus.0.elapsed(), focus.1.as_secs())).await?;
        }
    } else {
        msg.reply(ctx, "There was a problem getting the focus list").await?;
    }

    Ok(())
}

