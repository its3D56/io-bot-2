use random::Source;
use serenity::all as se;
use crate::message_helper::{
    new_message,
    reply,
};



pub fn reg() -> se::CreateCommand {
    se::CreateCommand::new("cat")
        .description("Sends a random picture of my cats")
}

pub async fn run(cmd: &se::CommandInteraction, ctx: &se::Context) {
	let cat_pictures: std::io::Result<Vec<std::fs::DirEntry>> = std::fs::read_dir("D:\\photos\\cats")
	    .and_then(|f| { f.collect() })
	;

	let Ok(cat_pictures) = cat_pictures else {
	    println!("WARN: Failed to get cat pictures");
	    return;
	};

    let time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|f| f.as_micros())
        .unwrap_or(0)
    ;

    let mut random_source = random::default(time as u64);
    
    let random_cat_pic = cat_pictures[random_source.read::<usize>() % cat_pictures.len()].path();
    let attachment = match se::CreateAttachment::path(&random_cat_pic).await {
        Ok(a) => a,
        Err(why) => {
            println!("WARN: Failed to create attachment: {why}");
            return;
        }
    };

    let message = new_message().add_file(attachment);
    
    if let Err(why) = reply(cmd, ctx, message).await {
        println!("WARN: Failed to reply with xkcd comic: {why}");
        return;
    }

    println!("INFO: Sent cat picture: {}", random_cat_pic.to_str().unwrap())
}
