use serenity::all as se;
use crate::message_helper::{
    new_message,
    reply,
};



pub fn reg() -> se::CreateCommand {
    se::CreateCommand::new("rps")
        .description("Rock paper scissors")
    
}

pub async fn run(cmd: &se::CommandInteraction, ctx: &se::Context) {
    	
}
