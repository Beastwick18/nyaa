use std::{cmp::max, error::Error};

use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use ratatui::{
    layout::{Alignment, Constraint},
    style::Stylize as _,
};
use reqwest::{StatusCode, Url};
use scraper::{ElementRef, Html, Selector};
use urlencoding::encode;

use crate::{
    app::{Context, Widgets},
    info,
    results::{ResultColumn, ResultHeader, ResultRow, ResultTable},
    theme::Theme,
    util::conv::{shorten_number, to_bytes},
    widget::sort::{SelectedSort, Sort},
};

use super::{add_protocol, Item, ItemType, Source, SourceInfo};

pub struct NyaaHtmlSource;

pub fn nyaa_table(items: Vec<Item>, theme: &Theme, sel_sort: &SelectedSort) -> ResultTable {
    let raw_date_width = items.iter().map(|i| i.date.len()).max().unwrap_or_default() as u16;
    let date_width = max(raw_date_width, 6);

    let header = ResultHeader::new([
        ResultColumn::Normal("Cat".to_owned(), Constraint::Length(3)),
        ResultColumn::Normal("Name".to_owned(), Constraint::Min(3)),
        ResultColumn::Sorted("Size".to_owned(), 9, Sort::Size as u32),
        ResultColumn::Sorted("Date".to_owned(), date_width, Sort::Date as u32),
        ResultColumn::Sorted("".to_owned(), 4, Sort::Seeders as u32),
        ResultColumn::Sorted("".to_owned(), 4, Sort::Leechers as u32),
        ResultColumn::Sorted("".to_owned(), 5, Sort::Downloads as u32),
    ]);
    let binding = header.get_binding();
    let align = [
        Alignment::Left,
        Alignment::Left,
        Alignment::Right,
        Alignment::Left,
        Alignment::Right,
        Alignment::Right,
        Alignment::Left,
    ];
    let rows: Vec<ResultRow> = items
        .iter()
        .map(|item| {
            ResultRow::new([
                item.icon.label.fg(item.icon.color),
                item.title.to_owned().fg(match item.item_type {
                    ItemType::Trusted => theme.trusted,
                    ItemType::Remake => theme.remake,
                    ItemType::None => theme.fg,
                }),
                item.size.clone().into(),
                item.date.clone().into(),
                item.seeders.to_string().fg(theme.trusted),
                item.leechers.to_string().fg(theme.remake),
                shorten_number(item.downloads).into(),
            ])
            .aligned(align, binding.to_owned())
            .fg(theme.fg)
        })
        .collect();

    ResultTable {
        headers: header.get_row(sel_sort.dir, sel_sort.sort as u32),
        rows,
        binding: header.get_binding(),
        items,
    }
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
    async fn search(
        client: &reqwest::Client,
        ctx: &mut Context,
        w: &Widgets,
    ) -> Result<ResultTable, Box<dyn Error>> {
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

        // let client = super::request_client(ctx)?;
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

        let items: Vec<Item> = doc
            .select(item_sel)
            .filter_map(|e| {
                let cat_str = attr(e, icon_sel, "href");
                let cat_str = cat_str.split('=').last().unwrap_or("");
                let cat = Self::info().entry_from_str(cat_str);
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
            .collect();

        // let raw_date_width = items.iter().map(|i| i.date.len()).max().unwrap_or_default() as u16;
        // let date_width = max(raw_date_width, 6);
        // let header = ResultHeader::new([
        //     ResultColumn::Normal("Cat".to_owned(), Constraint::Length(3)),
        //     ResultColumn::Normal("Name".to_owned(), Constraint::Min(3)),
        //     ResultColumn::Sorted("Size".to_owned(), 9),
        //     ResultColumn::Sorted("Date".to_owned(), date_width),
        //     ResultColumn::Sorted("".to_owned(), 4),
        //     ResultColumn::Sorted("".to_owned(), 4),
        //     ResultColumn::Sorted("".to_owned(), 5),
        // ]);
        // let s = header
        //     .get_render(w.sort.selected.dir)
        //     .iter()
        //     .fold("".to_owned(), |acc, s| format!("{}\"{}\"\n", acc, s));
        // ctx.show_error(s);
        Ok(nyaa_table(items, &ctx.theme, &w.sort.selected))
    }
    async fn sort(
        client: &reqwest::Client,
        ctx: &mut Context,
        w: &Widgets,
    ) -> Result<ResultTable, Box<dyn Error>> {
        NyaaHtmlSource::search(client, ctx, w).await
    }
    async fn filter(
        client: &reqwest::Client,
        ctx: &mut Context,
        w: &Widgets,
    ) -> Result<ResultTable, Box<dyn Error>> {
        NyaaHtmlSource::search(client, ctx, w).await
    }
    async fn categorize(
        client: &reqwest::Client,
        ctx: &mut Context,
        w: &Widgets,
    ) -> Result<ResultTable, Box<dyn Error>> {
        NyaaHtmlSource::search(client, ctx, w).await
    }

    fn info() -> SourceInfo {
        info! {
            "All Categories" => {
                0 => ("---", "All Categories", "AllCategories", White);
            }
            "Anime" => {
                10 => ("Ani", "All Anime", "AllAnime", Gray);
                12 => ("Sub", "English Translated", "AnimeEnglishTranslated", LightMagenta);
                13 => ("Sub", "Non-English Translated", "AnimeNonEnglishTranslated", LightGreen);
                14 => ("Raw", "Raw", "AnimeRaw", Gray);
                11 => ("AMV", "Anime Music Video", "AnimeMusicVideo", Magenta);
            }
            "Audio" => {
                20 => ("Aud", "All Audio", "AllAudio", Gray);
                21 => ("Aud", "Lossless", "AudioLossless", Red);
                22 => ("Aud", "Lossy", "AudioLossy", Yellow);
            }
            "Literature" => {
                30 => ("Lit", "All Literature", "AllLiterature", Gray);
                31 => ("Lit", "English Translated", "LitEnglishTranslated", LightGreen);
                32 => ("Lit", "Non-English Translated", "LitNonEnglishTranslated", Yellow);
                33 => ("Lit", "Raw", "LitRaw", Gray);
            }
            "Live Action" => {
                40 => ("Liv", "All Live Action", "AllLiveAction", Gray);
                41 => ("Liv", "English Translated", "LiveEnglishTranslated", Yellow);
                43 => ("Liv", "Non-English Translated", "LiveNonEnglishTranslated", LightCyan);
                42 => ("Liv", "Idol/Promo Video", "LiveIdolPromoVideo", LightYellow);
                44 => ("Liv", "Raw", "LiveRaw", Gray);
            }
            "Pictures" => {
                50 => ("Pic", "All Pictures", "AllPictures", Gray);
                51 => ("Pic", "Graphics", "PicGraphics", LightMagenta);
                52 => ("Pic", "Photos", "PicPhotos", Magenta);
            }
            "Software" => {
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
