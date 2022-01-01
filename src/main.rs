use std::{
    fs::{read_to_string, write},
    string::ToString,
    thread::sleep,
    time::Duration,
};

use frankenstein::{api_params::File, ChatId, SendPhotoParamsBuilder, TelegramApi};
use regex::Regex;
use scraper::Selector;

const SENT_FILE: &str = "sent.txt";

fn main() {
    let bot_token = std::env::var("BOT_TOKEN").expect("BOT_TOKEN is not set");
    let bot = frankenstein::Api::new(&bot_token);

    let chat_id = ChatId::Integer(-1_001_306_037_773); // https://telegram.me/LuicellasLangeReihe

    println!("Hello, world!");

    match get_picture_urls() {
        Ok(picture_pages) => {
            println!("found {} picture_pages", picture_pages.len());

            let mut already_sent = read_to_string(SENT_FILE)
                .unwrap_or_default()
                .lines()
                .filter(|o| !o.is_empty())
                .map(ToString::to_string)
                .collect::<Vec<_>>();

            for url in picture_pages {
                if !already_sent.contains(&url) {
                    println!("wait before continue");
                    sleep(Duration::from_secs(15));

                    match handle_picture_page(&bot, &chat_id, &url) {
                        Ok(_) => {
                            already_sent.push(url);
                        }
                        Err(err) => eprintln!("picture page ERROR {}", err),
                    }
                }
            }

            write(SENT_FILE, already_sent.join("\n") + "\n").unwrap();
        }
        Err(err) => eprintln!("ERROR: {}", err),
    }
}

const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64; rv:89.0) Gecko/20100101 Firefox/89.0";
fn get(url: &str) -> anyhow::Result<String> {
    let body = ureq::get(url)
        .set("USER-AGENT", USER_AGENT)
        .timeout(Duration::from_secs(5))
        .call()?
        .into_string()?;
    Ok(body)
}

fn get_picture_urls() -> anyhow::Result<Vec<String>> {
    let photo_page_regex =
        Regex::new(r#"[^"]+facebook.com\\/luicellaslangereihe\\/photos\\/a[^"]+"#).unwrap();
    let main_body = get("https://de-de.facebook.com/pg/luicellaslangereihe/photos/")?;
    let mut picture_pages = Vec::new();
    for c in photo_page_regex.captures_iter(&main_body) {
        let url = c[0]
            .replace("\\/", "/")
            .replace("www.facebook.com", "m.facebook.com");
        picture_pages.push(url);
    }

    // Newest Picture is at the top, so reverse them to have it at the end (→ handle oldest first)
    picture_pages.reverse();
    Ok(picture_pages)
}

fn handle_picture_page(
    bot: &frankenstein::Api,
    chat_id: &ChatId,
    page_url: &str,
) -> anyhow::Result<()> {
    println!("\nhandle_picture_page {}", page_url);
    let selector = Selector::parse("meta[property]").unwrap();

    let body = get(page_url)?;
    let html = scraper::Html::parse_document(&body);
    let metas = html.select(&selector).map(|o| o.html()).collect::<Vec<_>>();

    let image = find_content(&metas, "og:image")
        .ok_or_else(|| anyhow::anyhow!("image not found"))?
        .replace("&amp;", "&");
    let description = find_content(&metas, "og:description")
        .ok_or_else(|| anyhow::anyhow!("description not found"))?;
    let url = find_content(&metas, "og:url").ok_or_else(|| anyhow::anyhow!("url not found"))?;

    let send_photo_params = SendPhotoParamsBuilder::default()
        .chat_id(chat_id.clone())
        .photo(File::String(image))
        .parse_mode("Html")
        .caption(format!(
            r#"{}
<a href="{}">Quelle (Facebook)</a>"#,
            description, url
        ))
        .build()?;
    bot.send_photo(&send_photo_params)
        .map_err(|err| anyhow::anyhow!("{:?}", err))?;

    Ok(())
}

fn find_content<'s, S>(metas: S, property: &str) -> Option<&'s str>
where
    S: IntoIterator<Item = &'s String>,
{
    let content_regex = Regex::new(r#"content="([^"]+)"#).unwrap();
    let hit = metas.into_iter().find(|o| o.contains(property))?;
    let content = content_regex.captures(hit)?.get(1)?.as_str();
    Some(content)
}
