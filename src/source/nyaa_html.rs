use std::error::Error;

use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use reqwest::{StatusCode, Url};
use scraper::{ElementRef, Html, Selector};
use urlencoding::encode;

use crate::widget::category::Categories;

use crate::{
    app::{Context, Widgets},
    categories,
    util::conv::to_bytes,
};

use super::{add_protocol, Item, ItemType, Source};

pub struct NyaaHtmlSource;

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
    async fn search(ctx: &mut Context, w: &Widgets) -> Result<Vec<Item>, Box<dyn Error>> {
        let cat = ctx.category;
        let filter = w.filter.selected as u16;
        let page = ctx.page;
        let user = ctx.user.to_owned().unwrap_or_default();
        let sort = w.sort.selected.sort.to_url();

        let base_url = add_protocol(ctx.config.base_url.clone(), true);
        let (high, low) = (cat / 10, cat % 10);
        let query = encode(&w.search.input.input);
        let dir = w.sort.selected.dir.to_url();
        let url = Url::parse(&base_url)?;
        let mut url_query = url.clone();
        url_query.set_query(Some(&format!(
            "q={}&c={}_{}&f={}&p={}&s={}&o={}&u={}",
            query, high, low, filter, page, sort, dir, user
        )));

        let client = super::request_client(ctx)?;
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

        ctx.last_page = 100;
        ctx.total_results = 7500;
        // For searches, pagination has a description of total results found
        if let Some(pagination) = doc.select(pagination_sel).next() {
            // 6th word in pagination description contains total number of results
            if let Some(num_results_str) = pagination.inner_html().split(' ').nth(5) {
                if let Ok(num_results) = num_results_str.parse::<usize>() {
                    ctx.last_page = (num_results + 74) / 75;
                    ctx.total_results = num_results;
                }
            }
        }

        Ok(doc
            .select(item_sel)
            .filter_map(|e| {
                let cat_str = attr(e, icon_sel, "href");
                let cat_str = cat_str.split('=').last().unwrap_or("");
                let cat = NyaaHtmlSource::categories().entry_from_str(cat_str);
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
                let date = date_time.format(&ctx.config.date_format).to_string();

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

                let trusted = e.value().classes().any(|e| e == "success");
                let remake = e.value().classes().any(|e| e == "danger");
                let item_type = match (trusted, remake) {
                    (true, _) => ItemType::Trusted,
                    (_, true) => ItemType::Remake,
                    _ => ItemType::None,
                };

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
                    item_type,
                })
            })
            .collect())
    }
    async fn sort(ctx: &mut Context, w: &Widgets) -> Result<Vec<Item>, Box<dyn Error>> {
        NyaaHtmlSource::search(ctx, w).await
    }
    async fn filter(ctx: &mut Context, w: &Widgets) -> Result<Vec<Item>, Box<dyn Error>> {
        NyaaHtmlSource::search(ctx, w).await
    }
    async fn categorize(ctx: &mut Context, w: &Widgets) -> Result<Vec<Item>, Box<dyn Error>> {
        NyaaHtmlSource::search(ctx, w).await
    }

    fn categories() -> Categories {
        categories! {
            ALL_CATEGORIES;
            (ALL: "All Categories".to_owned()) => {
                0 => ("---", "All Categories", "AllCategories", White);
            }
            (ANIME: "Anime".to_owned()) => {
                10 => ("Ani", "All Anime", "AllAnime", Gray);
                12 => ("Sub", "English Translated", "AnimeEnglishTranslated", LightMagenta);
                13 => ("Sub", "Non-English Translated", "AnimeNonEnglishTranslated", LightGreen);
                14 => ("Raw", "Raw", "AnimeRaw", Gray);
                11 => ("AMV", "Anime Music Video", "AnimeMusicVideo", Magenta);
            }
            (AUDIO: "Audio".to_owned()) => {
                20 => ("Aud", "All Audio", "AllAudio", Gray);
                21 => ("Aud", "Lossless", "AudioLossless", Red);
                22 => ("Aud", "Lossy", "AudioLossy", Yellow);
            }
            (LITERATURE: "Literature".to_owned()) => {
                30 => ("Lit", "All Literature", "AllLiterature", Gray);
                31 => ("Lit", "English Translated", "LitEnglishTranslated", LightGreen);
                32 => ("Lit", "Non-English Translated", "LitNonEnglishTranslated", Yellow);
                33 => ("Lit", "Raw", "LitRaw", Gray);
            }
            (LIVE_ACTION: "Live Action".to_owned()) => {
                40 => ("Liv", "All Live Action", "AllLiveAction", Gray);
                41 => ("Liv", "English Translated", "LiveEnglishTranslated", Yellow);
                43 => ("Liv", "Non-English Translated", "LiveNonEnglishTranslated", LightCyan);
                42 => ("Liv", "Idol/Promo Video", "LiveIdolPromoVideo", LightYellow);
                44 => ("Liv", "Raw", "LiveRaw", Gray);
            }
            (PICTURES: "Pictures".to_owned()) => {
                50 => ("Pic", "All Pictures", "AllPictures", Gray);
                51 => ("Pic", "Graphics", "PicGraphics", LightMagenta);
                52 => ("Pic", "Photos", "PicPhotos", Magenta);
            }
            (SOFTWARE: "Software".to_owned()) => {
                60 => ("Sof", "All Software", "AllSoftware", Gray);
                61 => ("Sof", "Applications", "SoftApplications", Blue);
                62 => ("Sof", "Games", "SoftGames", LightBlue);
            }
        }
    }

    fn default_category() -> usize {
        0
    }
}
