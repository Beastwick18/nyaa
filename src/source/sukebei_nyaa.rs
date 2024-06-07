use std::{error::Error, time::Duration};

use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use ratatui::style::Color;
use reqwest::{StatusCode, Url};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use strum::VariantArray as _;
use urlencoding::encode;

use crate::{
    cats,
    results::ResultResponse,
    sel,
    sync::SearchQuery,
    theme::Theme,
    util::{
        conv::to_bytes,
        html::{attr, inner},
    },
};

use super::{
    add_protocol,
    nyaa_html::{nyaa_table, NyaaColumns, NyaaFilter, NyaaSort},
    nyaa_rss, Item, ItemType, ResultTable, Source, SourceConfig, SourceInfo, SourceResponse,
};

#[derive(Serialize, Deserialize, Clone, Copy, Default)]
#[serde(default)]
pub struct SukebeiTheme {
    #[serde(rename = "categories")]
    pub cat: SukebeiCategoryTheme,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
#[serde(default)]
pub struct SukebeiCategoryTheme {
    #[serde(with = "color_to_tui")]
    pub art_anime: Color,
    #[serde(with = "color_to_tui")]
    pub art_doujinshi: Color,
    #[serde(with = "color_to_tui")]
    pub art_games: Color,
    #[serde(with = "color_to_tui")]
    pub art_manga: Color,
    #[serde(with = "color_to_tui")]
    pub art_pictures: Color,
    #[serde(with = "color_to_tui")]
    pub real_photos: Color,
    #[serde(with = "color_to_tui")]
    pub real_videos: Color,
}

impl Default for SukebeiCategoryTheme {
    fn default() -> Self {
        use Color::*;
        Self {
            art_anime: Magenta,
            art_doujinshi: LightMagenta,
            art_games: Green,
            art_manga: LightGreen,
            art_pictures: Gray,
            real_photos: Red,
            real_videos: Yellow,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct SukebeiNyaaConfig {
    pub base_url: String,
    pub default_sort: NyaaSort,
    pub default_filter: NyaaFilter,
    pub default_category: String,
    pub default_search: String,
    pub rss: bool,
    pub timeout: Option<u64>,
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
            rss: false,
            timeout: None,
            columns: None,
        }
    }
}

pub struct SukebeiHtmlSource;

impl Source for SukebeiHtmlSource {
    async fn filter(
        client: &reqwest::Client,
        search: &SearchQuery,
        config: &SourceConfig,
        date_format: Option<String>,
    ) -> Result<SourceResponse, Box<dyn Error + Send + Sync>> {
        SukebeiHtmlSource::search(client, search, config, date_format).await
    }
    async fn categorize(
        client: &reqwest::Client,
        search: &SearchQuery,
        config: &SourceConfig,
        date_format: Option<String>,
    ) -> Result<SourceResponse, Box<dyn Error + Send + Sync>> {
        SukebeiHtmlSource::search(client, search, config, date_format).await
    }
    async fn sort(
        client: &reqwest::Client,
        search: &SearchQuery,
        config: &SourceConfig,
        date_format: Option<String>,
    ) -> Result<SourceResponse, Box<dyn Error + Send + Sync>> {
        let sukebei = config.sukebei.to_owned().unwrap_or_default();
        let sort = search.sort;
        let mut res = SukebeiHtmlSource::search(client, search, config, date_format).await;

        if sukebei.rss {
            if let Ok(SourceResponse::Results(res)) = &mut res {
                nyaa_rss::sort_items(&mut res.items, sort);
            }
        }
        res
    }

    async fn search(
        client: &reqwest::Client,
        search: &SearchQuery,
        config: &SourceConfig,
        date_format: Option<String>,
    ) -> Result<SourceResponse, Box<dyn Error + Send + Sync>> {
        let sukebei = config.sukebei.to_owned().unwrap_or_default();
        if sukebei.rss {
            return nyaa_rss::search_rss::<Self>(
                sukebei.base_url,
                sukebei.timeout,
                client,
                search,
                date_format,
            )
            .await;
        }
        let cat = search.category;
        let filter = search.filter;
        let page = search.page;
        let user = search.user.to_owned().unwrap_or_default();
        let sort = NyaaSort::from_repr(search.sort.sort)
            .unwrap_or(NyaaSort::Date)
            .to_url();

        let base_url = add_protocol(sukebei.base_url, true);
        let (high, low) = (cat / 10, cat % 10);
        let query = encode(&search.query);
        let dir = search.sort.dir.to_url();
        let url = Url::parse(&base_url)?;
        let mut url_query = url.clone();
        url_query.set_query(Some(&format!(
            "q={}&c={}_{}&f={}&p={}&s={}&o={}&u={}",
            query, high, low, filter, page, sort, dir, user
        )));

        let mut request = client.get(url_query.to_owned());
        if let Some(timeout) = sukebei.timeout {
            request = request.timeout(Duration::from_secs(timeout));
        }
        let response = request.send().await?;
        if response.status() != StatusCode::OK {
            // Throw error if response code is not OK
            let code = response.status().as_u16();
            return Err(format!("{}\nInvalid response code: {}", url_query, code).into());
        }
        let content = response.bytes().await?;
        let doc = Html::parse_document(std::str::from_utf8(&content[..])?);

        let item_sel = &sel!("table.torrent-list > tbody > tr")?;
        let icon_sel = &sel!("td:first-of-type > a")?;
        let title_sel = &sel!("td:nth-of-type(2) > a:last-of-type")?;
        let torrent_sel = &sel!("td:nth-of-type(3) > a:nth-of-type(1)")?;
        let magnet_sel = &sel!("td:nth-of-type(3) > a:nth-of-type(2)")?;
        let size_sel = &sel!("td:nth-of-type(4)")?;
        let date_sel = &sel!("td:nth-of-type(5)").unwrap();
        let seed_sel = &sel!("td:nth-of-type(6)")?;
        let leech_sel = &sel!("td:nth-of-type(7)")?;
        let dl_sel = &sel!("td:nth-of-type(8)")?;
        let pagination_sel = &sel!(".pagination-page-info")?;

        let mut last_page = 100;
        let mut total_results = 7500;
        // For searches, pagination has a description of total results found
        if let Some(pagination) = doc.select(pagination_sel).next() {
            // 6th word in pagination description contains total number of results
            if let Some(num_results_str) = pagination.inner_html().split(' ').nth(5) {
                if let Ok(num_results) = num_results_str.parse::<usize>() {
                    last_page = (num_results + 74) / 75;
                    total_results = num_results;
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
                    .map(Into::into)
                    .unwrap_or("null".to_owned());
                let id = post_link.split('/').last()?.parse::<usize>().ok()?;
                let id = format!("sukebei-{}", id);
                let file_name = format!("{}.torrent", id);

                let size = inner(e, size_sel, "0 B")
                    .replace('i', "")
                    .replace("Bytes", "B");
                let bytes = to_bytes(&size);

                let mut date = inner(e, date_sel, "");
                if let Some(date_format) = date_format.to_owned() {
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
                    .map(Into::into)
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
        Ok(SourceResponse::Results(ResultResponse {
            items,
            last_page,
            total_results,
        }))
        // Ok(nyaa_table(
        //     items,
        //     &theme,
        //     &search.sort,
        //     sukebei.columns,
        //     last_page,
        //     total_results,
        // ))
    }

    async fn solve(
        _solution: String,
        client: &reqwest::Client,
        search: &SearchQuery,
        config: &SourceConfig,
        date_format: Option<String>,
    ) -> Result<SourceResponse, Box<dyn Error + Send + Sync>> {
        SukebeiHtmlSource::search(client, search, config, date_format).await
    }

    fn info() -> SourceInfo {
        let cats = cats! {
            "All Categories" => {
                0 => ("---", "All Categories", "AllCategories", fg);
            }
            "Art" => {
                10 => ("Art", "All Art", "AllArt", fg);
                11 => ("Ani", "Anime", "ArtAnime", sukebei.cat.art_anime);
                12 => ("Dou", "Doujinshi", "ArtDoujinshi", sukebei.cat.art_doujinshi);
                13 => ("Gam", "Games", "ArtGames", sukebei.cat.art_games);
                14 => ("Man", "Manga", "ArtManga", sukebei.cat.art_manga);
                15 => ("Pic", "Pictures", "ArtPictures", sukebei.cat.art_pictures);
            }
            "Real Life" => {
                20 => ("Rea", "All Real Life", "AllReal", fg);
                21 => ("Pho", "Photobooks and Pictures", "RealPhotos", sukebei.cat.real_photos);
                22 => ("Vid", "Videos", "RealVideos", sukebei.cat.real_videos);
            }
        };
        SourceInfo {
            cats,
            filters: NyaaFilter::VARIANTS
                .iter()
                .map(ToString::to_string)
                .collect(),
            sorts: NyaaSort::VARIANTS.iter().map(ToString::to_string).collect(),
        }
    }

    fn load_config(config: &mut SourceConfig) {
        if config.sukebei.is_none() {
            config.sukebei = Some(SukebeiNyaaConfig::default());
        }
    }

    fn default_category(cfg: &SourceConfig) -> usize {
        let default = cfg
            .sukebei
            .as_ref()
            .map(|c| c.default_category.to_owned())
            .unwrap_or_default();
        Self::info().entry_from_cfg(&default).id
    }

    fn default_sort(cfg: &SourceConfig) -> usize {
        cfg.sukebei
            .as_ref()
            .map(|c| c.default_sort as usize)
            .unwrap_or_default()
    }

    fn default_filter(cfg: &SourceConfig) -> usize {
        cfg.sukebei
            .as_ref()
            .map(|c| c.default_filter as usize)
            .unwrap_or_default()
    }

    fn default_search(cfg: &SourceConfig) -> String {
        cfg.sukebei
            .as_ref()
            .map(|c| c.default_search.to_owned())
            .unwrap_or_default()
    }

    fn format_table(
        items: &[Item],
        search: &SearchQuery,
        config: &SourceConfig,
        theme: &Theme,
    ) -> ResultTable {
        let sukebei = config.sukebei.to_owned().unwrap_or_default();
        nyaa_table(items.into(), theme, &search.sort, &sukebei.columns)
    }
}
