use std::{cmp::Ordering, collections::BTreeMap, error::Error, str::FromStr};

use chrono::{DateTime, Local};
use reqwest::{StatusCode, Url};
use rss::{extension::Extension, Channel};
use urlencoding::encode;

use crate::{
    app::{Context, Widgets},
    util::conv::to_bytes,
    widget::sort::{SelectedSort, Sort, SortDir},
};

use super::{add_protocol, nyaa_html::NyaaHtmlSource, Item, ItemType, Source, SourceInfo};

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

fn sort_items(items: &mut [Item], sort: SelectedSort) {
    let f: fn(&Item, &Item) -> Ordering = match sort.sort {
        Sort::Date => |a, b| a.id.cmp(&b.id),
        Sort::Downloads => |a, b| b.downloads.cmp(&a.downloads),
        Sort::Seeders => |a, b| b.seeders.cmp(&a.seeders),
        Sort::Leechers => |a, b| b.leechers.cmp(&a.leechers),
        Sort::Size => |a, b| b.bytes.cmp(&a.bytes),
    };
    items.sort_by(f);
    if sort.dir == SortDir::Asc {
        items.reverse();
    }
}

impl Source for NyaaRssSource {
    async fn sort(
        _: &reqwest::Client,
        _: &mut Context,
        w: &Widgets,
    ) -> Result<Vec<Item>, Box<dyn Error>> {
        let mut items = w.results.table.items.clone();
        sort_items(&mut items, w.sort.selected);
        Ok(items)
    }

    async fn search(
        client: &reqwest::Client,
        ctx: &mut Context,
        w: &Widgets,
    ) -> Result<Vec<Item>, Box<dyn Error>> {
        let cat = ctx.category;
        let query = w.search.input.input.clone();
        let filter = w.filter.selected as usize;
        let user = ctx.user.to_owned().unwrap_or_default();
        ctx.last_page = 1;
        ctx.page = 1;
        let (high, low) = (cat / 10, cat % 10);
        let query = encode(&query);
        let base_url = add_protocol(ctx.config.base_url.clone(), true);
        let base_url = Url::parse(&base_url)?;

        let mut url = base_url.clone();
        let query = format!(
            "page=rss&f={}&c={}_{}&q={}&u={}&m",
            filter, high, low, query, user
        );
        url.set_query(Some(&query));

        // let client = super::request_client(ctx)?;

        let response = client.get(url.to_owned()).send().await?;
        let code = response.status().as_u16();
        if code != StatusCode::OK {
            // Throw error if response code is not OK
            return Err(format!("{}\nInvalid response code: {}", url, code).into());
        }

        let bytes = response.bytes().await?;
        let channel = Channel::read_from(&bytes[..])?;

        let mut results: Vec<Item> = channel
            .items
            .iter()
            .filter_map(|item| {
                let ext = item.extensions().get("nyaa")?;
                let guid = item.guid()?;
                let post = guid.value.clone();
                let id = guid.value.rsplit('/').next().unwrap_or_default(); // Get nyaa id from guid url in format
                                                                            // `https://nyaa.si/view/{id}`
                let id_usize = id.parse::<usize>().ok()?;
                let category_str = get_ext_value::<String>(ext, "categoryId");
                let cat = Self::info().entry_from_str(&category_str);
                let category = cat.id;
                let icon = cat.icon.clone();
                let size = get_ext_value::<String>(ext, "size")
                    .replace('i', "")
                    .replace("Bytes", "B");
                let pub_date = item.pub_date().unwrap_or("");
                let date = DateTime::parse_from_rfc2822(pub_date).unwrap_or_default();
                let date = date.with_timezone(&Local);
                let torrent_link = base_url
                    .join(&format!("/download/{}.torrent", id))
                    .map(|u| u.to_string())
                    .unwrap_or("null".to_owned());
                let trusted = get_ext_value::<String>(ext, "trusted").eq("Yes");
                let remake = get_ext_value::<String>(ext, "remake").eq("Yes");
                let item_type = match (trusted, remake) {
                    (true, _) => ItemType::Trusted,
                    (_, true) => ItemType::Remake,
                    _ => ItemType::None,
                };

                Some(Item {
                    id: id_usize,
                    date: date.format(&ctx.config.date_format).to_string(),
                    seeders: get_ext_value(ext, "seeders"),
                    leechers: get_ext_value(ext, "leechers"),
                    downloads: get_ext_value(ext, "downloads"),
                    bytes: to_bytes(&size),
                    size,
                    title: item.title().unwrap_or("???").to_owned(),
                    torrent_link,
                    magnet_link: item.link().unwrap_or("???").to_owned(),
                    post_link: post,
                    file_name: format!("{}.torrent", id),
                    item_type,
                    category,
                    icon,
                })
            })
            .collect();
        ctx.total_results = results.len();
        sort_items(&mut results, w.sort.selected);
        Ok(results)
    }

    async fn filter(
        client: &reqwest::Client,
        app: &mut Context,
        w: &Widgets,
    ) -> Result<Vec<Item>, Box<dyn Error>> {
        NyaaRssSource::search(client, app, w).await
    }

    async fn categorize(
        client: &reqwest::Client,
        app: &mut Context,
        w: &Widgets,
    ) -> Result<Vec<Item>, Box<dyn Error>> {
        NyaaRssSource::search(client, app, w).await
    }

    fn info() -> SourceInfo {
        NyaaHtmlSource::info()
    }

    fn default_category() -> usize {
        0
    }
}
