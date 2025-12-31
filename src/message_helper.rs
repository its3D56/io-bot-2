use serenity::all::{
    self as se,
    CreateInteractionResponseMessage,
};



pub fn new_message() -> CreateInteractionResponseMessage {
    CreateInteractionResponseMessage::new()
}

pub async fn reply(cmd: &se::CommandInteraction, ctx: &se::Context, message: CreateInteractionResponseMessage) -> Result<(), se::Error> {
    cmd.create_response(
        ctx, 
        se::CreateInteractionResponse::Message(message)
    ).await
}