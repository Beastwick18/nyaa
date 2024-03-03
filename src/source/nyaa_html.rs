use std::{error::Error, time::Duration};

use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use regex::Regex;
use reqwest::StatusCode;
use scraper::{Html, Selector};
use urlencoding::encode;

use crate::{
    app::{App, Widgets},
    widget::category::ALL_CATEGORIES,
};

use super::{Item, Source};

pub struct NyaaHtmlSource;

pub fn to_bytes(size: &str) -> usize {
    let mut split = size.split_whitespace();
    let b = split.next().unwrap_or("0");
    let unit = split.last().unwrap_or("B");
    let f = b.parse::<f64>().unwrap_or(0.0);
    let factor: i32 = match unit.chars().next().unwrap_or('B') {
        'T' => 12,
        'G' => 9,
        'M' => 6,
        'K' => 3,
        _ => 1,
    };
    (f64::powi(10.0, factor) * f).floor() as usize
}

impl Source for NyaaHtmlSource {
    async fn filter(app: &mut App, w: &Widgets) -> Result<Vec<Item>, Box<dyn Error>> {
        NyaaHtmlSource::search(app, w).await
    }
    async fn categorize(app: &mut App, w: &Widgets) -> Result<Vec<Item>, Box<dyn Error>> {
        NyaaHtmlSource::search(app, w).await
    }
    async fn sort(app: &mut App, w: &Widgets) -> Result<Vec<Item>, Box<dyn Error>> {
        NyaaHtmlSource::search(app, w).await
    }
    async fn search(app: &mut App, w: &Widgets) -> Result<Vec<Item>, Box<dyn Error>> {
        let cat = w.category.category;
        let filter = w.filter.selected.clone() as u16;
        let page = app.page;
        let sort = w.sort.selected.to_url();
        let timeout = app.config.timeout;

        let re = Regex::new(r"^https?://.+$")?;
        let base_url = match !re.is_match(&app.config.base_url) {
            // Assume https if not present
            true => format!("https://{}", app.config.base_url),
            false => app.config.base_url.to_owned(),
        };
        let (high, low) = (cat / 10, cat % 10);
        let query = encode(&w.search.input.input);
        let ord = match app.ascending {
            true => "asc",
            false => "desc",
        };
        let url = format!(
            "{}/?q={}&c={}_{}&f={}&p={}&s={}&o={}",
            base_url, query, high, low, filter, page, sort, ord
        );

        let client = reqwest::Client::builder()
            .gzip(true)
            .timeout(Duration::from_secs(timeout))
            .build()?;
        let response = client.get(url.to_owned()).send().await?;
        if response.status() != StatusCode::OK {
            // Throw error if response code is not OK
            let code = response.status().as_u16();
            return Err(format!("{}\nInvalid repsponse code: {}", url, code).into());
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
        let seeders_sel = &Selector::parse("td:nth-of-type(6)")?;
        let leechers_sel = &Selector::parse("td:nth-of-type(7)")?;
        let downloads_sel = &Selector::parse("td:nth-of-type(8)")?;
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

        let mut items: Vec<Item> = vec![];
        for (index, e) in doc.select(item_sel).enumerate() {
            let trusted = e.value().classes().any(|e| e == "success");
            let remake = e.value().classes().any(|e| e == "danger");
            let cat_str = e
                .select(icon_sel)
                .next()
                .and_then(|i| i.value().attr("href"))
                .and_then(|i| i.split('=').last())
                .unwrap_or("");
            let split: Vec<&str> = cat_str.split('_').collect();
            let high = split.first().unwrap_or(&"1").parse().unwrap_or(1);
            let low = split.last().unwrap_or(&"0").parse().unwrap_or(0);
            let category = high * 10 + low;
            let title = e
                .select(title_sel)
                .next()
                .and_then(|i| i.value().attr("title"))
                .unwrap_or("");
            let torrent_rel = e
                .select(torrent_sel)
                .next()
                .and_then(|i| i.value().attr("href"))
                .unwrap_or("");
            let magnet_link = e
                .select(magnet_sel)
                .next()
                .and_then(|i| i.value().attr("href"))
                .unwrap_or("");
            let size = e
                .select(size_sel)
                .next()
                .map(|i| i.inner_html())
                .unwrap_or("0 Bytes".to_owned())
                .replace('i', "")
                .replace("Bytes", "B");
            let bytes = to_bytes(&size);
            let date = e
                .select(date_sel)
                .next()
                .map(|i| i.inner_html())
                .unwrap_or("".to_owned());
            let naive = NaiveDateTime::parse_from_str(&date, "%Y-%m-%d %H:%M").unwrap_or_default();
            let date_time: DateTime<Local> = Local.from_utc_datetime(&naive);
            let date = date_time.format("%m/%d/%y %H:%M").to_string();
            let seeders = e
                .select(seeders_sel)
                .next()
                .map(|i| i.inner_html())
                .unwrap_or("0".to_owned())
                .parse::<u32>()
                .unwrap_or(0);
            let leechers = e
                .select(leechers_sel)
                .next()
                .map(|i| i.inner_html())
                .unwrap_or("0".to_owned())
                .parse::<u32>()
                .unwrap_or(0);
            let downloads = e
                .select(downloads_sel)
                .next()
                .map(|i| i.inner_html())
                .unwrap_or("0".to_owned())
                .parse::<u32>()
                .unwrap_or(0);
            let file_name = torrent_rel.split('/').last().unwrap_or("nyaa.torrent");

            let icon = ALL_CATEGORIES
                .get(high)
                .and_then(|c| c.find(category))
                .unwrap_or_default();
            items.push(Item {
                index,
                date,
                seeders,
                leechers,
                downloads,
                size,
                bytes,
                title: title.to_owned(),
                torrent_link: format!("{}{}", base_url, torrent_rel),
                magnet_link: magnet_link.to_owned(),
                file_name: file_name.to_owned(),
                category,
                icon,
                trusted,
                remake,
            });
        }
        Ok(items)
    }
}
