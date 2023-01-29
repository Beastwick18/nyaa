use core::str::FromStr;
use num_derive::FromPrimitive;
use rss::{extension::Extension, Channel};
use std::{
    collections::BTreeMap,
    error::Error,
    slice::Iter,
    string::ToString,
};
use tui::{
    style::{Color, Style},
    text::Text,
};
use urlencoding::encode;
use serde::{Deserialize, Serialize};

pub mod config;

type ExtensionMap = BTreeMap<String, Vec<Extension>>;

pub struct Item {
    pub index: u32,
    pub seeders: u32,
    pub leechers: u32,
    pub downloads: u32,
    pub title: String,
    pub torrent_link: String,
    pub category: Category,
    pub trusted: bool,
    pub remake: bool,
}

impl Item {
    pub fn get_styled_title(&self) -> Text {
        if self.trusted {
            return Text::styled(self.title.to_owned(), Style::default().fg(Color::Green));
        } else if self.remake {
            return Text::styled(self.title.to_owned(), Style::default().fg(Color::Red));
        }
        Text::from(self.title.to_owned())
    }
}

pub trait EnumIter<T> {
    fn iter() -> Iter<'static, T>;
}

#[derive(Copy, Clone, FromPrimitive, PartialEq, Deserialize, Serialize)]
#[allow(clippy::enum_variant_names)]
pub enum Filter {
    NoFilter = 0,
    NoRemakes = 1,
    TrustedOnly = 2,
}

#[derive(Copy, Clone, FromPrimitive, PartialEq, Deserialize, Serialize)]
pub enum Category {
    AllAnime = 0,
    EnglishTranslated = 2,
    NonEnglishTranslated = 3,
    Raw = 4,
    AnimeMusicVideo = 1,
}

#[derive(Copy, Clone, FromPrimitive, PartialEq, Deserialize, Serialize)]
pub enum Sort {
    Date = 0,
    Downloads = 1,
    Seeders = 2,
    Leechers = 3,
    Name = 4,
    Category = 5,
}

impl Category {
    fn get_url_string(&self) -> String {
        "1_".to_owned() + &(self.to_owned() as i32).to_string()
    }

    pub fn get_icon(&self) -> Text {
        match self {
            Category::AllAnime => Text::raw(""),
            Category::AnimeMusicVideo => {
                Text::styled("AMV", Style::default().fg(Color::Magenta))
            }
            Category::EnglishTranslated => {
                Text::styled("Subs", Style::default().fg(Color::Magenta))
            }
            Category::NonEnglishTranslated => {
                Text::styled("Subs", Style::default().fg(Color::Green))
            }
            Category::Raw => {
                Text::styled("Raw", Style::default().fg(Color::Gray))
            }
        }
    }
}

impl EnumIter<Category> for Category {
    fn iter() -> Iter<'static, Category> {
        static CATEGORIES: [Category; 5] = [
            Category::AllAnime,
            Category::EnglishTranslated,
            Category::NonEnglishTranslated,
            Category::Raw,
            Category::AnimeMusicVideo,
        ];
        CATEGORIES.iter()
    }
}

impl ToString for Category {
    fn to_string(&self) -> String {
        match self {
            Category::AllAnime => "All Anime",
            Category::AnimeMusicVideo => "Anime Music Video",
            Category::EnglishTranslated => "English Translated",
            Category::NonEnglishTranslated => "Non-English Translated",
            Category::Raw => "Raw",
        }.to_owned()
    }
}

impl Default for Category {
    fn default() -> Category {
        Category::AllAnime
    }
}

impl Filter {
    fn get_url_string(&self) -> String {
        (self.to_owned() as i32).to_string()
    }
}

impl EnumIter<Filter> for Filter {
    fn iter() -> Iter<'static, Filter> {
        static FILTERS: [Filter; 3] = [Filter::NoFilter, Filter::NoRemakes, Filter::TrustedOnly];
        FILTERS.iter()
    }
}

impl ToString for Filter {
    fn to_string(&self) -> String {
        match self {
            Filter::NoFilter => "No Filter",
            Filter::NoRemakes => "No Remakes",
            Filter::TrustedOnly => "Trusted Only",
        }.to_owned()
    }
}

impl Default for Filter {
    fn default() -> Filter {
        Filter::NoFilter
    }
}

impl EnumIter<Sort> for Sort {
    fn iter() -> Iter<'static, Sort> {
        static SORTS: [Sort; 6] = [
            Sort::Date,
            Sort::Downloads,
            Sort::Seeders,
            Sort::Leechers,
            Sort::Name,
            Sort::Category,
        ];
        SORTS.iter()
    }
}

impl ToString for Sort {
    fn to_string(&self) -> String {
        match self {
            Sort::Date => "Date",
            Sort::Downloads => "Downloads",
            Sort::Seeders => "Seeders",
            Sort::Leechers => "Leechers",
            Sort::Name => "Name",
            Sort::Category => "Category",
        }.to_owned()
    }
}

impl Default for Sort {
    fn default() -> Sort {
        Sort::Date
    }
}

pub async fn get_feed_list(query: &String, cat: &Category, filter: &Filter, magnet: bool) -> Vec<Item> {
    let feed = match get_feed(query.to_owned(), cat, filter, magnet).await {
        Ok(x) => x,
        Err(_) => panic!("Failed to connect to nyaa.si..."),
    };
    let mut items: Vec<Item> = Vec::new();

    for (i, item) in feed.items.iter().enumerate() {
        if let (Some(ext_map), Some(title), Some(link)) =
            (item.extensions().get("nyaa"), &item.title, &item.link)
        {
            let seeders = get_ext_value::<u32>(ext_map, "seeders")
                .await
                .unwrap_or_default();
            let leechers = get_ext_value(ext_map, "leechers").await.unwrap_or_default();
            let downloads = get_ext_value(ext_map, "downloads")
                .await
                .unwrap_or_default();
            let category_str: String = get_ext_value::<String>(ext_map, "categoryId")
                .await
                .unwrap_or_default();
            let trusted: bool = get_ext_value::<String>(ext_map, "trusted")
                .await
                .unwrap_or_default()
                .eq("Yes");
            let remake: bool = get_ext_value::<String>(ext_map, "remake")
                .await
                .unwrap_or_default()
                .eq("Yes");

            items.push(Item {
                index: i as u32,
                seeders,
                leechers,
                downloads,
                title: title.to_owned(),
                torrent_link: link.to_owned(),
                category: num::FromPrimitive::from_u32(
                    category_str.chars().last().unwrap() as u32 - '0' as u32,
                )
                .unwrap(),
                trusted,
                remake,
            });
        }
    }
    items
}

pub async fn get_feed(
    query: String,
    cat: &Category,
    filter: &Filter,
    magnet: bool
) -> Result<Channel, Box<dyn Error>> {
    let m = if magnet { "&m" } else { "" };
    let encoded_url = format!(
        "https://nyaa.si/?page=rss&f={}&c={}&q={}{}",
        filter.get_url_string(),
        cat.get_url_string(),
        encode(&query),
        m
    );
    let content = reqwest::get(encoded_url).await?.bytes().await?;

    let channel = Channel::read_from(&content[..])?;
    Ok(channel)
}

pub async fn get_ext_value<T>(ext_map: &ExtensionMap, key: &str) -> Option<T>
where
    T: FromStr,
{
    if let Some(seeders) = ext_map.get(key) {
        if let Some(seeders2) = seeders.get(0) {
            if let Some(seeder_value) = &seeders2.value {
                return match seeder_value.to_string().parse::<T>() {
                    Ok(x) => Some(x),
                    Err(_) => None,
                };
            }
        }
    }
    None
}
