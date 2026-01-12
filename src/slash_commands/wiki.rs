use serenity::all as se;
use crate::message_helper::{
    new_message,
    reply,
};



const USER_AGENT: &str = "Io's discord bot/1.0 (its3d56@gmail.com)";



pub fn reg() -> se::CreateCommand {
    se::CreateCommand::new("wiki")
        .description("Look something up on wikipedia")
        .add_option(
            se::CreateCommandOption::new(se::CommandOptionType::String, "query", "Search query")
            .required(true)
        )
}

pub async fn run(cmd: &se::CommandInteraction, ctx: &se::Context) {
	let option = cmd
	    .data
	    .options()
	    .pop()
	    .expect("Discord api should always provide command option")
	    .value	   
	; 

	let se::ResolvedValue::String(query) = option else {
	    unreachable!("Should always be a string")
	};
    	
    let reqwest_client = reqwest::ClientBuilder::new()
        .user_agent(USER_AGENT)
        .build()
        .unwrap()
    ;

    let page = find_page(query, &reqwest_client).await;
	
	let message_content = match page {
	    Ok(ref page) => page.get_formatted(),
	    _ => format!("Could not find article \"{query}\""),
	};

	let message = new_message().content(truncate_text(message_content));

	if let Err(why) = reply(cmd, ctx, message).await {
	    println!("WARN: Failed to reply with wikipedia article: {why}");
	    return;
	}

	match page {
	    Ok(page) => println!("INFO: Replied with article: {}", page.title),
	    Err(why) => println!("INFO: Failed to get article \"{query}\": {why}")
	}
}



fn truncate_text(mut text: String) -> String {
    const MESSAGE_MAX_LEN: usize = 2000;
    let len = text.len();
    if len <= MESSAGE_MAX_LEN {
        return text;
    }
    
    text.truncate(MESSAGE_MAX_LEN - 3);
    text.push_str("...");
    text
}

type PageTitle = String;

async fn find_page(query: &str, reqwest_client: &reqwest::Client) -> WikiResult<Page> {
    let title = search_wikipedia(query, reqwest_client).await?;
    get_page(&title, reqwest_client).await
}

pub async fn search_wikipedia(query: &str, reqwest_client: &reqwest::Client) -> WikiResult<PageTitle> {
    let query = urlencoding::encode(query);
    let request_url = format!("https://en.wikipedia.org/w/api.php?format=json&action=opensearch&limit=1&search={query}");
    let response_raw = reqwest_client.get(request_url)
        .send()
        .await?
        .text()
        .await?
    ;
    
    let response_json = json::parse(&response_raw)?;
    if response_json[1].is_empty() {
       return Err(Box::new(WikiError::PageNotFound)); 
    }
    
    let title = &response_json[1][0]; 
    let Some(title) = title.as_str() else {
        return Err(Box::new(WikiError::InvalidResponse));
    };

    Ok(title.to_string())
}

async fn get_page(page_title: &PageTitle, reqwest_client: &reqwest::Client) -> WikiResult<Page> {
    let page_title = urlencoding::encode(page_title);
    let request_url = format!("https://en.wikipedia.org/w/api.php?format=json&action=query&prop=extracts|pageprops|links&exintro&explaintext&redirects=1&titles={page_title}");

    let response_raw = reqwest_client.get(request_url)
        .send()
        .await?
        .text()
        .await?
    ;
   
    let response_json = json::parse(&response_raw)?;
    Page::from_json(response_json)
}

struct Page {
    title: PageTitle,
    is_disambiguation: bool,
    links: Vec<String>,
    extract: String,
}

impl Page {
    fn from_json(json: json::JsonValue) -> WikiResult<Self> {
        let Some((_, page)) = json["query"]["pages"].entries().next_back() else {
            return Err(Box::new(WikiError::InvalidResponse));
        };

        let Some(title) = page["title"].as_str().map(str::to_string) else {
            return Err(Box::new(WikiError::InvalidResponse));
        };
        
        let is_disambiguation = page["pageprops"].has_key("disambiguation");
        
        let Some(extract) = page["extract"].as_str().map(str::to_string) else {
            return Err(Box::new(WikiError::InvalidResponse));
        };

        let links = page["links"].members().filter_map(
            |link| link["title"].as_str().map(str::to_string)
        ).collect();

        Ok(Self {
            title,
            is_disambiguation,
            links,
            extract,
        })
    }

    fn get_formatted(&self) -> String {
        let title = &self.title;
        if self.is_disambiguation {
            format!("# '{}' May Refer to:\n{}", &title, self.links.join("\n"))
        } else {
            format!("# {}\n{}", &title, self.extract)
        }
    }
}

#[derive(Debug)]
enum WikiError {
    InvalidResponse,
    PageNotFound,
}

impl std::fmt::Display for WikiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
         f.write_str(match self {
            WikiError::InvalidResponse => "Invalid response from wikipedia api.",
            WikiError::PageNotFound => "Page not found.",
        })
    }
}

impl std::error::Error for WikiError {}

type WikiResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;
