use std::{error::Error, time::Duration};

use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use reqwest::{StatusCode, Url};
use scraper::{ElementRef, Html, Selector};
use urlencoding::encode;

use crate::{
    app::{Context, Widgets},
    widget::category::CatEntry,
};

use super::{add_protocol, Item, Source};

pub struct NyaaHtmlSource;

pub fn to_bytes(size: &str) -> usize {
    let mut split = size.split_whitespace();
    let b = split.next().unwrap_or("0");
    let unit = split.last().unwrap_or("B");
    let f = b.parse::<f64>().unwrap_or(0.0);
    let power = match unit.chars().next().unwrap_or('B') {
        'T' => 12,
        'G' => 9,
        'M' => 6,
        'K' => 3,
        _ => 1,
    };
    (f64::powi(10.0, power) * f) as usize
}

fn inner(e: ElementRef, s: &Selector, default: &str) -> String {
    e.select(s)
        .next()
        .map(|i| i.inner_html())
        .unwrap_or(default.to_owned())
}

fn attr(e: ElementRef, s: &Selector, attr: &str) -> String {
    e.select(s)
        .next()
        .and_then(|i| i.value().attr(attr))
        .unwrap_or("")
        .to_owned()
}

impl Source for NyaaHtmlSource {
    async fn filter(app: &mut Context, w: &Widgets) -> Result<Vec<Item>, Box<dyn Error>> {
        NyaaHtmlSource::search(app, w).await
    }
    async fn categorize(app: &mut Context, w: &Widgets) -> Result<Vec<Item>, Box<dyn Error>> {
        NyaaHtmlSource::search(app, w).await
    }
    async fn sort(app: &mut Context, w: &Widgets) -> Result<Vec<Item>, Box<dyn Error>> {
        NyaaHtmlSource::search(app, w).await
    }
    async fn search(app: &mut Context, w: &Widgets) -> Result<Vec<Item>, Box<dyn Error>> {
        let cat = w.category.category;
        let filter = w.filter.selected as u16;
        let page = app.page;
        let sort = w.sort.selected.to_url();
        let timeout = app.config.timeout;

        let base_url = add_protocol(app.config.base_url.clone(), true);
        let (high, low) = (cat / 10, cat % 10);
        let query = encode(&w.search.input.input);
        let ord = match app.ascending {
            true => "asc",
            false => "desc",
        };
        let url = Url::parse(&base_url)?;
        let mut url_query = url.clone();
        url_query.set_query(Some(&format!(
            "q={}&c={}_{}&f={}&p={}&s={}&o={}",
            query, high, low, filter, page, sort, ord
        )));

        let client = reqwest::Client::builder()
            .gzip(true)
            .timeout(Duration::from_secs(timeout))
            .build()?;
        let response = client.get(url_query.to_owned()).send().await?;
        if response.status() != StatusCode::OK {
            // Throw error if response code is not OK
            let code = response.status().as_u16();
            return Err(format!("{}\nInvalid repsponse code: {}", url_query, code).into());
        }
        let content = response.bytes().await?;
        let doc = Html::parse_document(std::str::from_utf8(&content[..])?);

        let item_sel = &Selector::parse("table.torrent-list > tbody > tr")?;
        let icon_sel = &Selector::parse("td:first-of-type > a")?;
        let title_sel = &Selector::parse("td:nth-of-type(2) > a:last-of-type")?;
        let torrent_sel = &Selector::parse("td:nth-of-type(3) > a:nth-of-type(1)")?;
        let magnet_sel = &Selector::parse("td:nth-of-type(3) > a:nth-of-type(2)")?;
        let size_sel = &Selector::parse("td:nth-of-type(4)")?;
        let date_sel = &Selector::parse("td:nth-of-type(5)").unwrap();
        let seed_sel = &Selector::parse("td:nth-of-type(6)")?;
        let leech_sel = &Selector::parse("td:nth-of-type(7)")?;
        let dl_sel = &Selector::parse("td:nth-of-type(8)")?;
        let pagination_sel = &Selector::parse(".pagination-page-info")?;

        app.last_page = 100;
        app.total_results = 7500;
        // For searches, pagination has a description of total results found
        if let Some(pagination) = doc.select(pagination_sel).next() {
            // 6th word in pagination description contains total number of results
            if let Some(num_results_str) = pagination.inner_html().split(' ').nth(5) {
                if let Ok(num_results) = num_results_str.parse::<usize>() {
                    app.last_page = (num_results + 74) / 75;
                    app.total_results = num_results;
                }
            }
        }

        Ok(doc
            .select(item_sel)
            .filter_map(|e| {
                let cat_str = attr(e, icon_sel, "href");
                let cat_str = cat_str.split('=').last().unwrap_or("");
                let cat = CatEntry::from_str(cat_str);
                let category = cat.id;
                let icon = cat.icon.clone();

                let torrent = attr(e, torrent_sel, "href");
                let id = torrent
                    .split('/')
                    .last()?
                    .split('.')
                    .next()?
                    .parse::<usize>()
                    .ok()?;
                let file_name = format!("{}.torrent", id);

                let size = inner(e, size_sel, "0 bytes")
                    .replace('i', "")
                    .replace("Bytes", "B");
                let bytes = to_bytes(&size);

                let date = inner(e, date_sel, "");
                let naive =
                    NaiveDateTime::parse_from_str(&date, "%Y-%m-%d %H:%M").unwrap_or_default();
                let date_time: DateTime<Local> = Local.from_utc_datetime(&naive);
                let date = date_time.format(&app.config.date_format).to_string();

                let seeders = inner(e, seed_sel, "0").parse().unwrap_or(0);
                let leechers = inner(e, leech_sel, "0").parse().unwrap_or(0);
                let downloads = inner(e, dl_sel, "0").parse().unwrap_or(0);
                let torrent_link = url
                    .join(&torrent)
                    .map(|u| u.to_string())
                    .unwrap_or("null".to_owned());
                let post_link = url
                    .join(&attr(e, title_sel, "href"))
                    .map(|url| url.to_string())
                    .unwrap_or("null".to_owned());

                Some(Item {
                    id,
                    date,
                    seeders,
                    leechers,
                    downloads,
                    size,
                    bytes,
                    title: attr(e, title_sel, "title"),
                    torrent_link,
                    magnet_link: attr(e, magnet_sel, "href"),
                    post_link,
                    file_name: file_name.to_owned(),
                    category,
                    icon,
                    trusted: e.value().classes().any(|e| e == "success"),
                    remake: e.value().classes().any(|e| e == "danger"),
                })
            })
            .collect())
    }
}
