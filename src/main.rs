use std::{
    fs::{read_to_string, write},
    string::ToString,
    time::Duration,
};

use frankenstein::{ChatIdEnum, FileEnum, SendPhotoParams, TelegramApi};
use regex::Regex;

const SENT_FILE: &str = "sent.txt";

fn main() {
    let bot_token = std::env::var("BOT_TOKEN").expect("BOT_TOKEN is not set");
    let bot = frankenstein::Api::new(bot_token);

    let chat_id = ChatIdEnum::IsizeVariant(-1_001_306_037_773); // https://telegram.me/LuicellasLangeReihe
    println!("Hello, world!");

    match get_picture_urls() {
        Ok(pictures) => {
            let mut already_sent = read_to_string(SENT_FILE)
                .unwrap_or_default()
                .lines()
                .filter(|o| !o.is_empty())
                .map(ToString::to_string)
                .collect::<Vec<_>>();

            for p in pictures {
                if !already_sent.contains(&p) {
                    println!("unknown pic {}", p);
                    bot.send_photo(&SendPhotoParams::new(
                        chat_id.clone(),
                        FileEnum::StringVariant(p.to_string()),
                    ))
                    .unwrap();
                    already_sent.push(p);
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
        Regex::new(r#"https:\\/\\/www.facebook.com\\/luicellaslangereihe\\/photos\\/a[^"]+"#)
            .unwrap();
    let content_regex = Regex::new(r#""contentUrl":"([^"]+)"#).unwrap();

    let main_body = get("https://de-de.facebook.com/pg/luicellaslangereihe/photos/")?;
    let mut pictures = Vec::new();
    for bla in photo_page_regex.captures_iter(&main_body) {
        let url = &bla[0]
            .replace("\\/", "/")
            .replace("www.facebook.com", "m.facebook.com");

        match handle_each_picture_page(&content_regex, url) {
            Ok(mut pics) => pictures.append(&mut pics),
            Err(err) => eprintln!("picture page ERROR {}", err),
        }
    }

    // Newest Picture is at the top, so reverse them to have it at the end (→ handle oldest first)
    pictures.reverse();
    Ok(pictures)
}

fn handle_each_picture_page(re: &Regex, url: &str) -> anyhow::Result<Vec<String>> {
    println!("get image from picture page {}", url);
    let hits = re
        .captures_iter(&get(url)?)
        .map(|o| o[1].replace("\\/", "/"))
        .collect::<Vec<_>>();
    Ok(hits)
}
