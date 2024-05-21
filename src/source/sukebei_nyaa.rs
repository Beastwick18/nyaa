use std::error::Error;

use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use reqwest::{StatusCode, Url};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use urlencoding::encode;

use crate::{
    app::{Context, Widgets},
    cats,
    config::Config,
    util::{
        conv::to_bytes,
        html::{attr, inner},
    },
    widget::EnumIter as _,
};

use super::{
    add_protocol,
    nyaa_html::{nyaa_table, NyaaColumns, NyaaFilter, NyaaSort},
    Item, ItemType, ResultTable, Source, SourceInfo,
};

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct SukebeiNyaaConfig {
    pub base_url: String,
    pub default_sort: NyaaSort,
    pub default_filter: NyaaFilter,
    pub default_category: String,
    pub default_search: String,
    pub columns: Option<NyaaColumns>,
}

impl Default for SukebeiNyaaConfig {
    fn default() -> Self {
        Self {
            base_url: "https://sukebei.nyaa.si/".to_owned(),
            default_sort: NyaaSort::Date,
            default_filter: NyaaFilter::NoFilter,
            default_category: "AllCategories".to_owned(),
            default_search: Default::default(),
            columns: None,
        }
    }
}

pub struct SubekiHtmlSource;

impl Source for SubekiHtmlSource {
    async fn filter(
        client: &reqwest::Client,
        ctx: &mut Context,
        w: &Widgets,
    ) -> Result<ResultTable, Box<dyn Error>> {
        SubekiHtmlSource::search(client, ctx, w).await
    }
    async fn categorize(
        client: &reqwest::Client,
        ctx: &mut Context,
        w: &Widgets,
    ) -> Result<ResultTable, Box<dyn Error>> {
        SubekiHtmlSource::search(client, ctx, w).await
    }
    async fn sort(
        client: &reqwest::Client,
        ctx: &mut Context,
        w: &Widgets,
    ) -> Result<ResultTable, Box<dyn Error>> {
        SubekiHtmlSource::search(client, ctx, w).await
    }
    async fn search(
        client: &reqwest::Client,
        ctx: &mut Context,
        w: &Widgets,
    ) -> Result<ResultTable, Box<dyn Error>> {
        let sukebei = ctx.config.sources.sukebei.to_owned().unwrap_or_default();
        let cat = w.category.selected;
        let filter = w.filter.selected as u16;
        let page = ctx.page;
        let user = ctx.user.to_owned().unwrap_or_default();
        let sort = NyaaSort::try_from(w.sort.selected.sort)
            .unwrap_or(NyaaSort::Date)
            .to_url();

        let base_url = add_protocol(sukebei.base_url, true);
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
                let post_link = url
                    .join(&attr(e, title_sel, "href"))
                    .map(|url| url.to_string())
                    .unwrap_or("null".to_owned());
                let id = post_link.split('/').last()?.parse::<usize>().ok()?;
                let file_name = format!("{}.torrent", id);

                let size = inner(e, size_sel, "0 B")
                    .replace('i', "")
                    .replace("Bytes", "B");
                let bytes = to_bytes(&size);

                let mut date = inner(e, date_sel, "");
                if let Some(date_format) = ctx.config.date_format.to_owned() {
                    let naive =
                        NaiveDateTime::parse_from_str(&date, "%Y-%m-%d %H:%M").unwrap_or_default();
                    let date_time: DateTime<Local> = Local.from_utc_datetime(&naive);
                    date = date_time.format(&date_format).to_string();
                }

                let seeders = inner(e, seed_sel, "0").parse().unwrap_or(0);
                let leechers = inner(e, leech_sel, "0").parse().unwrap_or(0);
                let downloads = inner(e, dl_sel, "0").parse().unwrap_or(0);
                let torrent_link = url
                    .join(&torrent)
                    .map(|u| u.to_string())
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
                    ..Default::default()
                })
            })
            .collect();
        Ok(nyaa_table(
            items,
            &ctx.theme,
            &w.sort.selected,
            sukebei.columns,
        ))
    }

    fn info() -> SourceInfo {
        let cats = cats! {
            "All Categories" => {
                0 => ("---", "All Categories", "AllCategories", White);
            }
            "Art" => {
                10 => ("Art", "All Art", "AllArt", Gray);
                11 => ("Ani", "Anime", "ArtAnime", Magenta);
                12 => ("Dou", "Doujinshi", "ArtDoujinshi", LightMagenta);
                13 => ("Gam", "Games", "ArtGames", LightMagenta);
                14 => ("Man", "Manga", "ArtManga", LightGreen);
                15 => ("Pic", "Pictures", "ArtPictures", Gray);
            }
            "Real Life" => {
                20 => ("Rea", "All Real Life", "AllReal", Gray);
                21 => ("Pho", "Photobooks and Pictures", "RealPhotos", Red);
                22 => ("Vid", "Videos", "RealVideos", Yellow);
            }
        };
        SourceInfo {
            cats,
            filters: NyaaFilter::iter().map(|f| f.to_string()).collect(),
            sorts: NyaaSort::iter().map(|item| item.to_string()).collect(),
        }
    }

    fn load_config(ctx: &mut Context) {
        if ctx.config.sources.sukebei.is_none() {
            ctx.config.sources.sukebei = Some(SukebeiNyaaConfig::default());
        }
    }

    fn default_category(cfg: &Config) -> usize {
        let default = cfg
            .sources
            .sukebei
            .to_owned()
            .unwrap_or_default()
            .default_category;
        Self::info().entry_from_cfg(&default).id
    }

    fn default_sort(cfg: &Config) -> usize {
        cfg.sources
            .sukebei
            .to_owned()
            .unwrap_or_default()
            .default_sort as usize
    }

    fn default_filter(cfg: &Config) -> usize {
        cfg.sources
            .sukebei
            .to_owned()
            .unwrap_or_default()
            .default_filter as usize
    }
}
