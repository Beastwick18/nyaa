use std::{cmp::Ordering, collections::BTreeMap, error::Error, str::FromStr, time::Duration};

use chrono::{DateTime, Local};
use regex::Regex;
use rss::{extension::Extension, Channel};
use urlencoding::encode;

use crate::{
    app::{App, Widgets},
    widget::{category::CatEntry, sort::Sort},
};

use super::{nyaa_html::to_bytes, Item, Source};

pub struct NyaaRssSource;

type ExtensionMap = BTreeMap<String, Vec<Extension>>;

pub fn get_ext_value<T: Default + FromStr>(ext_map: &ExtensionMap, key: &str) -> T {
    ext_map
        .get(key)
        .and_then(|v| v.first())
        .and_then(|s| s.value())
        .and_then(|val| val.parse().ok())
        .unwrap_or_default()
}

fn sort_items(items: &mut [Item], sort: Sort, ascending: bool) {
    let f: fn(&Item, &Item) -> Ordering = match sort {
        Sort::Date => |a, b| a.index.cmp(&b.index),
        Sort::Downloads => |a, b| b.downloads.cmp(&a.downloads),
        Sort::Seeders => |a, b| b.seeders.cmp(&a.seeders),
        Sort::Leechers => |a, b| b.leechers.cmp(&a.leechers),
        Sort::Size => |a, b| b.bytes.cmp(&a.bytes),
    };
    items.sort_by(f);
    if ascending {
        items.reverse();
    }
}

impl Source for NyaaRssSource {
    async fn sort(app: &mut App, w: &Widgets) -> Result<Vec<Item>, Box<dyn Error>> {
        let mut items = w.results.table.items.clone();
        sort_items(&mut items, w.sort.selected.clone(), app.ascending);
        Ok(items)
    }

    async fn search(app: &mut App, w: &Widgets) -> Result<Vec<Item>, Box<dyn Error>> {
        let cat = w.category.category;
        let query = w.search.input.input.clone();
        let filter = w.filter.selected.clone() as usize;
        app.last_page = 1;
        app.page = 1;
        let (high, low) = (cat / 10, cat % 10);
        let query = encode(&query);
        let mut base_url = app.config.base_url.clone();
        let re = Regex::new(r"^https?://.+$").unwrap();
        if !re.is_match(&base_url) {
            // Assume https if not present
            base_url = format!("https://{}", base_url);
        }

        let url = format!(
            "{}/?page=rss&f={}&c={}_{}&q={}&m",
            base_url, filter, high, low, query
        );
        let content = reqwest::get(url.clone()).await?.bytes().await?;

        let client = reqwest::Client::builder()
            .gzip(true)
            .timeout(Duration::from_secs(app.config.timeout))
            .build()?;
        let response = client.get(url.to_owned()).send().await?;
        let code = response.status().as_u16();
        if code != 200 {
            // Throw error if response code is not OK
            return Err(format!("{}\nInvalid response code: {}", url, code).into());
        }

        let channel = Channel::read_from(&content[..])?;

        let mut results: Vec<Item> = channel
            .items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| {
                let ext = item.extensions().get("nyaa")?;
                let guid = item.guid()?;
                let id = guid.value.rsplit('/').next().unwrap_or_default(); // Get nyaa id from guid url in format
                                                                            // `https://nyaa.si/view/{id}`
                let category_str = get_ext_value::<String>(ext, "categoryId");
                let cat = CatEntry::from_str(&category_str);
                let category = cat.id;
                let icon = cat.icon.clone();
                let size = get_ext_value::<String>(ext, "size")
                    .replace('i', "")
                    .replace("Bytes", "B");
                let pub_date = item.pub_date().unwrap_or("");
                let date = DateTime::parse_from_rfc2822(pub_date).unwrap_or_default();
                let date = date.with_timezone(&Local);

                Some(Item {
                    index,
                    date: date.format(&app.config.date_format).to_string(),
                    seeders: get_ext_value(ext, "seeders"),
                    leechers: get_ext_value(ext, "leechers"),
                    downloads: get_ext_value(ext, "downloads"),
                    bytes: to_bytes(&size),
                    size,
                    title: item.title().unwrap_or("???").to_owned(),
                    torrent_link: format!("{}/download/{}.torrent", base_url, id),
                    magnet_link: item.link().unwrap_or("???").to_owned(),
                    file_name: format!("{}.torrent", id),
                    trusted: get_ext_value::<String>(ext, "trusted").eq("Yes"),
                    remake: get_ext_value::<String>(ext, "remake").eq("Yes"),
                    category,
                    icon,
                })
            })
            .collect();
        app.total_results = results.len();
        sort_items(&mut results, w.sort.selected.clone(), app.ascending);
        Ok(results)
    }

    async fn filter(app: &mut App, w: &Widgets) -> Result<Vec<Item>, Box<dyn Error>> {
        NyaaRssSource::search(app, w).await
    }

    async fn categorize(app: &mut App, w: &Widgets) -> Result<Vec<Item>, Box<dyn Error>> {
        NyaaRssSource::search(app, w).await
    }
}
