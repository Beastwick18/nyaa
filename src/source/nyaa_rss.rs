use std::{cmp::Ordering, collections::BTreeMap, error::Error, str::FromStr, time::Duration};

use chrono::{DateTime, Local};
use reqwest::{StatusCode, Url};
use rss::{extension::Extension, Channel};
use urlencoding::encode;

use crate::{
    results::ResultResponse,
    sync::SearchQuery,
    util::conv::to_bytes,
    widget::sort::{SelectedSort, SortDir},
};

use super::{add_protocol, nyaa_html::NyaaSort, Item, ItemType, Source, SourceResponse};

type ExtensionMap = BTreeMap<String, Vec<Extension>>;

pub fn get_ext_value<T: Default + FromStr>(ext_map: &ExtensionMap, key: &str) -> T {
    ext_map
        .get(key)
        .and_then(|v| v.first())
        .and_then(|s| s.value())
        .and_then(|val| val.parse().ok())
        .unwrap_or_default()
}

pub fn sort_items(items: &mut [Item], sort: SelectedSort) {
    let f: fn(&Item, &Item) -> Ordering = match NyaaSort::from_repr(sort.sort) {
        Some(NyaaSort::Downloads) => |a, b| b.downloads.cmp(&a.downloads),
        Some(NyaaSort::Seeders) => |a, b| b.seeders.cmp(&a.seeders),
        Some(NyaaSort::Leechers) => |a, b| b.leechers.cmp(&a.leechers),
        Some(NyaaSort::Size) => |a, b| b.bytes.cmp(&a.bytes),
        _ => |a, b| a.id.cmp(&b.id),
    };
    items.sort_by(f);
    if sort.dir == SortDir::Asc {
        items.reverse();
    }
}

pub async fn search_rss<S: Source>(
    base_url: String,
    timeout: Option<u64>,
    client: &reqwest::Client,
    search: &SearchQuery,
    date_format: Option<String>,
) -> Result<SourceResponse, Box<dyn Error + Send + Sync>> {
    // let nyaa = config.nyaa.to_owned().unwrap_or_default();
    let query = search.query.to_owned();
    let cat = search.category;
    let filter = search.filter;
    let user = search.user.to_owned().unwrap_or_default();
    let last_page = 1;
    let (high, low) = (cat / 10, cat % 10);
    let query = encode(&query);
    let base_url = add_protocol(base_url, true);
    let base_url = Url::parse(&base_url)?;

    let mut url = base_url.clone();
    let query = format!(
        "page=rss&f={}&c={}_{}&q={}&u={}&m",
        filter, high, low, query, user
    );
    url.set_query(Some(&query));

    // let client = super::request_client(ctx)?;

    let mut request = client.get(url.to_owned());
    if let Some(timeout) = timeout {
        request = request.timeout(Duration::from_secs(timeout));
    }
    let response = request.send().await?;
    let code = response.status().as_u16();
    if code != StatusCode::OK {
        // Throw error if response code is not OK
        return Err(format!("{}\nInvalid response code: {}", url, code).into());
    }

    let bytes = response.bytes().await?;
    let channel = Channel::read_from(&bytes[..])?;

    let mut items: Vec<Item> = channel
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
            let cat = S::info().entry_from_str(&category_str);
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
                .map(Into::into)
                .unwrap_or("null".to_owned());
            let trusted = get_ext_value::<String>(ext, "trusted").eq("Yes");
            let remake = get_ext_value::<String>(ext, "remake").eq("Yes");
            let item_type = match (trusted, remake) {
                (true, _) => ItemType::Trusted,
                (_, true) => ItemType::Remake,
                _ => ItemType::None,
            };
            let date_format = date_format
                .to_owned()
                .unwrap_or("%Y-%m-%d %H:%M".to_owned());

            Some(Item {
                id: format!("nyaa-{}", id_usize),
                date: date.format(&date_format).to_string(),
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
                ..Default::default()
            })
        })
        .collect();
    let total_results = items.len();
    sort_items(&mut items, search.sort);
    Ok(SourceResponse::Results(ResultResponse {
        items,
        last_page,
        total_results,
    }))
    // Ok(items)
    // Ok(nyaa_table(
    //     items,
    //     &theme,
    //     &search.sort,
    //     nyaa.columns,
    //     last_page,
    //     total_results,
    // ))
}
