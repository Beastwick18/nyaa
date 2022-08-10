use core::str::FromStr;
use urlencoding::encode;
use rss::{Channel, extension::Extension};
use std::error::Error;
use log::debug;
use std::collections::BTreeMap;

pub mod config;

type ExtensionMap = BTreeMap<String, Vec<Extension>>;

pub struct Item {
    pub seeders: u32,
    pub leechers: u32,
    pub downloads: u32,
    pub title: String,
    pub torrent_link: String
}

enum Filters {
    NoFilter = 0,
    NoRemakes = 1,
    TrustedOnly = 2
}

// enum MainCategories {
//     Anime = 1 << 4,
//     Audio = 2 << 4,
//     Literature = 3 << 4,
//     LiveAction = 4 << 4,
//     Pictures = 5 << 4,
//     Software = 6 << 4
// }

#[derive(Copy, Clone)]
enum Categories {
    AllAnime = 0,
    AnimeMusicVideo = 1,
    EnglishTranslated = 2,
    NonEnglishTranslated = 3,
    Raw = 4
}

impl Categories {
    fn get_url_string(self) -> String {
        "1_".to_owned() + &(self as i32).to_string()
    }
    
    fn get_name(self) -> String {
        match self {
            Categories::AllAnime => "All Anime".to_string(),
            Categories::AnimeMusicVideo => "Anime Music Video".to_string(),
            Categories::EnglishTranslated => "English Translated".to_string(),
            Categories::NonEnglishTranslated => "Non-English Translated".to_string(),
            Categories::Raw => "Raw".to_string()
        }
    }
}

pub async fn get_feed(query: String) -> Result<Channel, Box<dyn Error>> {
    let encoded_url = format!("https://nyaa.si/?page=rss&f=0&c=1_2&q={}", encode(&query));
    debug!("{}", encoded_url);
    let content = reqwest::get(encoded_url)
        .await?
        .bytes()
        .await?;
    
    let channel = Channel::read_from(&content[..])?;
    Ok(channel)
}

pub async fn get_ext_value<T>(ext_map: &ExtensionMap, key: &str) -> Option<T> where T: FromStr {
    if let Some(seeders) = ext_map.get(key) {
        if let Some(seeders2) = seeders.get(0) {
            if let Some(seeder_value) = &seeders2.value {
                return match seeder_value.to_string().parse::<T>() {
                    Ok(x) => Some(x),
                    Err(_) => None
                }
            }
        }
    }
    None
}
