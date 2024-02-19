use std::{collections::BTreeMap, error::Error, str::FromStr};

use rss::{extension::Extension, Channel};
use urlencoding::encode;

use crate::widget::category::{CatIcon, ALL_CATEGORIES};

type ExtensionMap = BTreeMap<String, Vec<Extension>>;

pub struct Item {
    pub index: usize,
    pub seeders: u32,
    pub leechers: u32,
    pub downloads: u32,
    pub size: String,
    pub title: String,
    pub torrent_link: String,
    pub magnet_link: String,
    pub file_name: String,
    pub category: usize,
    pub icon: CatIcon,
    pub trusted: bool,
    pub remake: bool,
}

pub async fn get_feed_list(
    query: &String,
    cat: usize,
    filter: usize,
) -> Result<Vec<Item>, Box<dyn Error>> {
    let (high, low) = (cat / 10, cat % 10);
    let query = encode(&query);
    let url = format!(
        "https://nyaa.si/?page=rss&f={}&c={}_{}&q={}&m",
        filter, high, low, query
    );
    let content = reqwest::get(url).await?.bytes().await?;

    let channel = Channel::read_from(&content[..])?;

    Ok(channel
        .items
        .iter()
        .enumerate()
        .filter_map(|(index, item)| {
            let ext = item.extensions().get("nyaa")?;
            let guid = item.guid.to_owned().unwrap_or_default();
            let id = guid.value.split("/").last().unwrap_or_default().to_owned(); // Get nyaa id from guid url in format
                                                                                  // `https://nyaa.si/view/{id}`
            let category_str = get_ext_value::<String>(ext, "categoryId");
            let split: Vec<&str> = category_str.split("_").collect();
            let high = split.first().unwrap_or(&"1").parse::<usize>().unwrap_or(1);
            let low = split.last().unwrap_or(&"0").parse::<usize>().unwrap_or(0);
            let category = high * 10 + low;
            let icon = ALL_CATEGORIES
                .get(high - 1)
                .and_then(|c| c.find(category))
                .unwrap_or_default();

            Some(Item {
                index,
                seeders: get_ext_value(ext, "seeders"),
                leechers: get_ext_value(ext, "leechers"),
                downloads: get_ext_value(ext, "downloads"),
                size: get_ext_value::<String>(ext, "size").replace("i", ""),
                title: item.title.to_owned().unwrap_or("???".to_owned()),
                torrent_link: format!("https://nyaa.si/download/{}.torrent", id),
                magnet_link: item.link.to_owned().unwrap_or("???".to_owned()),
                file_name: format!("{}.torrent", id),
                trusted: get_ext_value::<String>(ext, "trusted").eq("Yes"),
                remake: get_ext_value::<String>(ext, "remake").eq("Yes"),
                category,
                icon,
            })
        })
        .collect())
}

pub fn get_ext_value<T: Default + FromStr>(ext_map: &ExtensionMap, key: &str) -> T {
    ext_map
        .get(key)
        .and_then(|v| v.get(0))
        .and_then(|s| s.value.to_owned())
        .and_then(|val| val.parse::<T>().ok())
        .unwrap_or_default()
}
