use std::{
    fs::{read_to_string, write},
    string::ToString,
    thread::sleep,
    time::Duration,
};

use frankenstein::{ChatIdEnum, FileEnum, SendPhotoParams, TelegramApi};
use regex::Regex;

const SENT_FILE: &str = "sent.txt";

fn main() {
    let bot_token = std::env::var("BOT_TOKEN").expect("BOT_TOKEN is not set");
    let bot = frankenstein::Api::new(bot_token);

    let chat_id = ChatIdEnum::IntegerVariant(-1_001_306_037_773); // https://telegram.me/LuicellasLangeReihe

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

fn get(url: &str) -> anyhow::Result<String> {
    let body = ureq::get(url)
        .set(
            "USER-AGENT",
            "Mozilla/5.0 (X11; Linux x86_64; rv:89.0) Gecko/20100101 Firefox/89.0",
        )
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
    chat_id: &ChatIdEnum,
    page_url: &str,
) -> anyhow::Result<()> {
    println!("\nhandle_picture_page {}", page_url);
    let body = get(page_url)?;
    let urls = Regex::new(r#""contentUrl":"([^"]+)"#)
        .unwrap()
        .captures_iter(&body)
        .map(|o| o[1].replace("\\/", "/"))
        .collect::<Vec<_>>();

    for url in urls {
        println!("wait then send to telegram chat... {}", url);
        sleep(Duration::from_secs(15));

        let mut send_photo_params =
            SendPhotoParams::new(chat_id.clone(), FileEnum::StringVariant(url.to_string()));
        send_photo_params.set_parse_mode(Some("Html".to_string()));
        send_photo_params.set_caption(Some(format!(
            r#"<a href="{}">Quelle (Facebook)</a>"#,
            page_url
        )));
        bot.send_photo(&send_photo_params)
            .map_err(|err| anyhow::anyhow!("{:?}", err))?;
    }

    Ok(())
}
