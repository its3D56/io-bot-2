mod slash_commands;
mod message_helper;
use std::sync::Arc;
use serenity::{
    all as se,
    prelude::*,
};



#[tokio::main]
async fn main() {
    let token = std::fs::read_to_string("token.txt")
        .expect("Failed to get token")
    ;

    let gateway_intents = 
        GatewayIntents::GUILDS |
        GatewayIntents::GUILD_EMOJIS_AND_STICKERS
    ;

    let mut client = serenity::Client::builder(token, gateway_intents)
        .event_handler(EventHandler)
        .await
        .expect("Failed to create client")
    ;

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
    }
    
    client
        .start()
        .await
        .expect("Failed to start client")
    ;    
}



struct ShardManagerContainer;

impl serenity::prelude::TypeMapKey for ShardManagerContainer {
    type Value = Arc<se::ShardManager>;
}

struct EventHandler;

#[serenity::async_trait]
impl serenity::client::EventHandler for EventHandler {
    async fn ready(&self, ctx: Context, _ready: se::Ready) {
        let command_count = register_commands(&ctx).await;

        if command_count == 1 {
            println!("INFO: Registered 1 command!")
        } else {
            println!("INFO: Registered {command_count} command(s)!");
        }
      
        println!("INFO: Bot started!")
    }

    async fn interaction_create(&self, ctx: Context, interaction: se::Interaction) {
        #[allow(clippy::single_match)]
        match interaction {
            se::Interaction::Command(cmd) => slash_command_used(&ctx, &cmd).await,
            _ => (),
        }
    }
}

type CommandCount = usize;
async fn register_commands(ctx: &Context) -> CommandCount {
    use slash_commands::*;
    
    let commands = [
        ping::reg(),
        xkcd::reg(),
        wiki::reg(),
        cat::reg(),
    ];

    let count = commands.len();

    for command in commands {
        if let Err(why) = se::Command::create_global_command(ctx, command).await {
            println!("WARN: Could not create global command: {why}")
        }
    }

    count
}

async fn slash_command_used(ctx: &Context, cmd: &se::CommandInteraction) {
    use slash_commands::*;
    
    let name = &cmd.data.name;
    let command_pretty = pretty_print_command(cmd);

    println!("INFO: Slash command used: {command_pretty}");

    match name.as_str() {
        "ping" => ping::run(cmd, ctx).await,
        "xkcd" => xkcd::run(cmd, ctx).await,
        "wiki" => wiki::run(cmd, ctx).await,
        "cat"  => cat::run(cmd, ctx).await,
        n => println!("WARN: No handler function for command: {n}"),
    }
}

fn pretty_print_command(cmd: &se::CommandInteraction) -> String {
    let options = cmd.data.options();
    let name = &cmd.data.name;

    if options.is_empty() {
        return name.to_string()
    }
    
    let formatted_options = options
        .iter()
        .map(format_option)
        .collect::<Vec<String>>()
        .join(" ")
    ;

    format!("{name} {formatted_options}")
}

fn format_option(opt: &se::ResolvedOption) -> String {
    use serenity::all::ResolvedValue as ResVal;
    
    let name = &opt.name;
    match opt.value {
        ResVal::SubCommand(_) => name.to_string(),
        ResVal::String(s) => format!("\"{s}\""),
        _ => unimplemented!(), // Unimplemented becasue we arent using any other option types
    }
}
