use crate::ShardManagerContainer;
use serenity::all as se;
use crate::message_helper::{
    new_message,
    reply,
};



pub fn reg() -> se::CreateCommand {
    se::CreateCommand::new("ping")
        .description("ping the bot")
}

pub async fn run(cmd: &se::CommandInteraction, ctx: &se::Context) {
    let latency = get_latency(ctx).await;
    let message_content = match latency {
        Some(v) => format!("Pong! Latency: {v}ms"),
        None => {
            println!("WARN: Failed to get latency");
            "Pong! Failed to get latency".into()
        },
    };

    let message = new_message().content(message_content);
    
    if let Err(why) = reply(cmd, ctx, message).await {
        println!("WARN: Failed to create interaction response: {why}");
        return;
    }

    if let Some(latency) = latency {
        println!("INFO: Replied with latency {latency}ms")
    }
}



async fn get_latency(ctx: &se::Context) -> Option<u128> {
    let data = ctx.data.read().await;
    
    let Some(shard_manager) = data.get::<ShardManagerContainer>() else {
        println!("WARN: Failed to get shard manager");
        return None;
    };

    let runners = shard_manager.runners.lock().await;

    let Some(runner) = runners.get(&ctx.shard_id) else {
        println!("WARN: No shard found");
        return None;
    };

    let Some(latency) = runner.latency else {
        println!("WARN: Failed to get latency");
        return None;
    };

    Some(latency.as_millis())
}