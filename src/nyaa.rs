use std::{collections::BTreeMap, error::Error, str::FromStr};

use rss::{extension::Extension, Channel};
use urlencoding::encode;

use crate::widget::category::{CatIcon, ALL_CATEGORIES};

type ExtensionMap = BTreeMap<String, Vec<Extension>>;

pub struct Item {
    pub index: u32,
    pub seeders: u32,
    pub leechers: u32,
    pub downloads: u32,
    pub title: String,
    pub torrent_link: String,
    pub magnet_link: String,
    pub file_name: String,
    pub id: String,
    pub hash: String,
    pub category: u32,
    pub icon: CatIcon,
    pub trusted: bool,
    pub remake: bool,
}

pub async fn get_feed_list(
    query: &String,
    cat: usize,
    filter: usize,
) -> Result<Vec<Item>, Box<dyn Error>> {
    let feed = get_feed(query.to_owned(), cat, filter, true).await?;
    let mut items: Vec<Item> = Vec::new();

    for (i, item) in feed.items.iter().enumerate() {
        if let (Some(ext_map), Some(title), Some(link), Some(guid)) = (
            item.extensions().get("nyaa"),
            &item.title,
            &item.link,
            &item.guid,
        ) {
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
            let hash: String = get_ext_value::<String>(ext_map, "infoHash")
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
            let id = guid.value.split("/").last().unwrap_or_default().to_owned(); // Get nyaa id from guid url in format
                                                                                  // `https://nyaa.si/view/{id}`
            let torrent_link = format!("https://nyaa.si/download/{}.torrent", id);
            let file_name = format!("{}.torrent", id);
            let split: Vec<&str> = category_str.split("_").collect();
            let high_str = match split.first() {
                Some(c) => c,
                None => "1",
            };
            let high = match high_str.parse::<u32>() {
                Ok(h) => h,
                Err(_) => 1,
            };
            let low_str = match split.last() {
                Some(c) => c,
                None => "1",
            };
            let low = match low_str.parse::<u32>() {
                Ok(l) => l,
                Err(_) => 1,
            };
            let category = high * 10 + low;
            let mut icon = ALL_CATEGORIES[0].entries[0].icon.clone();
            for cat in ALL_CATEGORIES {
                if let Some(i) = cat.find(category) {
                    icon = i;
                }
            }

            items.push(Item {
                index: i as u32,
                seeders,
                leechers,
                downloads,
                title: title.to_owned(),
                torrent_link,
                magnet_link: link.to_owned(),
                file_name,
                id,
                hash,
                category,
                icon,
                trusted,
                remake,
            });
        }
    }
    Ok(items)
}

pub async fn get_feed(
    query: String,
    cat: usize,
    filter: usize,
    magnet: bool,
) -> Result<Channel, Box<dyn Error>> {
    let m = if magnet { "&m" } else { "" };
    let encoded_url = format!(
        "https://nyaa.si/?page=rss&f={}&c={}_{}&q={}{}",
        filter,
        cat / 10,
        cat % 10,
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
