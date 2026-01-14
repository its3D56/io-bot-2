use std::time::UNIX_EPOCH;
use json::JsonValue;
use serenity::all as se;
use crate::message_helper::{
    new_message,
    reply,
};
use random::Source;



pub fn reg() -> se::CreateCommand {
    se::CreateCommand::new("xkcd")
        .description("Fetches an xkcd comic")
        .add_option(
            se::CreateCommandOption::new(se::CommandOptionType::SubCommand, "newest", "Fetches the latest xkcd comic")
        )
        .add_option(
            se::CreateCommandOption::new(se::CommandOptionType::SubCommand, "random", "Fetches a random xkcd comic")
        )
}

pub async fn run(cmd: &se::CommandInteraction, ctx: &se::Context) {
    let subcommand = cmd.data.options().pop().expect("Discord api should always provide subcommand");
    
    let comic = match subcommand.name {
        "newest" => get_newest_comic().await,
        "random" => get_random_comic().await,
        _ => unreachable!("Discord api should always provide valid subcommand")
    };
    
    let comic = match comic {
        Ok(r) => r,
        Err(why) => {
            println!("WARN: Failed to get xkcd comic: {why}");
            return;
        }
    };

    let attachment = match se::CreateAttachment::url(&ctx.http, &comic.img_url).await {
        Ok(r) => r,
        Err(why) => {
            println!("WARN: Failed to embed xkcd comic: {why}");
            return;
        }
    };
    
    let attachment = attachment.description(comic.transcript);

    let message_content = format!("# XKCD #{}: {}\n{}", comic.number, comic.title, comic.alt);
    
    let message = new_message()
        .content(message_content)
        .add_file(attachment)
    ;

    if let Err(why) = reply(cmd, ctx, message).await {
        println!("WARN: Failed to reply with xkcd comic: {why}");
        return;
    }

    println!("INFO: Sent comic #{}: \"{}\"", comic.number, comic.title)
}



async fn get_newest_comic() -> XKCDResult<ComicInfo> {
    get_comic("https://xkcd.com/info.0.json").await
}

async fn get_random_comic() -> XKCDResult<ComicInfo> {
    let newest_comic = get_newest_comic().await?;
    let time = std::time::SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let mut random_source = random::default(time as u64);
    let comic_num = random_source.read::<u32>() % newest_comic.number + 1;

    let comic_url = format!("https://xkcd.com/{comic_num}/info.0.json");
    get_comic(&comic_url).await
}

async fn get_comic(url: &str) -> XKCDResult<ComicInfo> {
    let response_raw = reqwest::get(url)
        .await?
        .text()
        .await?
    ;

    let json = json::parse(&response_raw)?;

    let Some(comic) = ComicInfo::from_json(json) else {
        return Err(Box::new(XKCDError::InvalidResponse));
    };

    Ok(comic)
}



type XKCDResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

struct ComicInfo {
    number: u32,
    title: String,
    alt: String,
    img_url: String,
    transcript: String,
}

impl ComicInfo {
    fn from_json(json: JsonValue) -> Option<Self> {
        Some(ComicInfo {
            number: json.get("num")?.as_u32()?,
            title: json.get("title")?.to_string(),
            alt: json.get("alt")?.to_string(),
            img_url: json.get("img")?.to_string(),
            transcript: json.get("transcript")?.to_string(),        
        })
    }
}

trait Get {
    type Content;
    fn get<'a>(&'a self, idx: &str) -> Option<&'a Self::Content>;
}

impl Get for JsonValue {
    type Content = JsonValue;
    fn get<'a>(&'a self, idx: &str) -> Option<&'a Self::Content> {
        match &self[idx] {
            JsonValue::Null => None,
            content => Some(content),
        }
    }
}

#[derive(Debug)]
enum XKCDError {
    InvalidResponse
}

impl std::fmt::Display for XKCDError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid response from xkcd api")
    }
}

impl std::error::Error for XKCDError {}
