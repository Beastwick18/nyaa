use std::{cmp::max, error::Error, time::Duration};

use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use ratatui::{
    layout::{Alignment, Constraint},
    style::Stylize as _,
};
use reqwest::{StatusCode, Url};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use urlencoding::encode;

use crate::{
    cats, cond_vec, popup_enum,
    results::{ResultColumn, ResultHeader, ResultResponse, ResultRow, ResultTable},
    sel,
    sync::SearchQuery,
    theme::Theme,
    util::{
        conv::{shorten_number, to_bytes},
        html::{attr, inner},
    },
    widget::{sort::SelectedSort, EnumIter as _},
};

use super::{add_protocol, nyaa_rss, Item, ItemType, Source, SourceConfig, SourceInfo};

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct NyaaConfig {
    pub base_url: String,
    pub default_sort: NyaaSort,
    pub default_filter: NyaaFilter,
    pub default_category: String,
    pub default_search: String,
    pub rss: bool,
    pub timeout: Option<u64>,
    pub columns: Option<NyaaColumns>,
}

#[derive(Clone, Copy, Serialize, Deserialize, Default)]
pub struct NyaaColumns {
    category: Option<bool>,
    title: Option<bool>,
    size: Option<bool>,
    date: Option<bool>,
    seeders: Option<bool>,
    leechers: Option<bool>,
    downloads: Option<bool>,
}

impl NyaaColumns {
    fn array(self) -> [bool; 7] {
        [
            self.category.unwrap_or(true),
            self.title.unwrap_or(true),
            self.size.unwrap_or(true),
            self.date.unwrap_or(true),
            self.seeders.unwrap_or(true),
            self.leechers.unwrap_or(true),
            self.downloads.unwrap_or(true),
        ]
    }
}

impl Default for NyaaConfig {
    fn default() -> Self {
        Self {
            base_url: "https://nyaa.si/".to_owned(),
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

popup_enum! {
    NyaaSort;
    (0, Date, "Date");
    (1, Downloads, "Downloads");
    (2, Seeders, "Seeders");
    (3, Leechers, "Leechers");
    (4, Size, "Size");
}

impl NyaaSort {
    pub fn to_url(self) -> String {
        match self {
            NyaaSort::Date => "id".to_owned(),
            NyaaSort::Downloads => "downloads".to_owned(),
            NyaaSort::Seeders => "seeders".to_owned(),
            NyaaSort::Leechers => "leechers".to_owned(),
            NyaaSort::Size => "size".to_owned(),
        }
    }
}

popup_enum! {
    NyaaFilter;
    #[allow(clippy::enum_variant_names)]
    (0, NoFilter, "No Filter");
    (1, NoRemakes, "No Remakes");
    (2, TrustedOnly, "Trusted Only");
    (3, Batches, "Batches");
}

pub struct NyaaHtmlSource;

pub fn nyaa_table(
    items: &[Item],
    theme: &Theme,
    sel_sort: &SelectedSort,
    columns: &Option<NyaaColumns>,
) -> ResultTable {
    let raw_date_width = items.iter().map(|i| i.date.len()).max().unwrap_or_default() as u16;
    let date_width = max(raw_date_width, 6);

    let header = ResultHeader::new([
        ResultColumn::Normal("Cat".to_owned(), Constraint::Length(3)),
        ResultColumn::Normal("Name".to_owned(), Constraint::Min(3)),
        ResultColumn::Sorted("Size".to_owned(), 9, NyaaSort::Size as u32),
        ResultColumn::Sorted("Date".to_owned(), date_width, NyaaSort::Date as u32),
        ResultColumn::Sorted("".to_owned(), 4, NyaaSort::Seeders as u32),
        ResultColumn::Sorted("".to_owned(), 4, NyaaSort::Leechers as u32),
        ResultColumn::Sorted("".to_owned(), 5, NyaaSort::Downloads as u32),
    ]);
    let mut binding = header.get_binding();
    let align = [
        Alignment::Left,
        Alignment::Left,
        Alignment::Right,
        Alignment::Left,
        Alignment::Right,
        Alignment::Right,
        Alignment::Left,
    ];
    let mut rows: Vec<ResultRow> = items
        .iter()
        .map(|item| {
            ResultRow::new([
                item.icon.label.fg(item.icon.color),
                item.title.to_owned().fg(match item.item_type {
                    ItemType::Trusted => theme.trusted,
                    ItemType::Remake => theme.remake,
                    ItemType::None => theme.fg,
                }),
                item.size.clone().fg(theme.fg),
                item.date.clone().fg(theme.fg),
                item.seeders.to_string().fg(theme.trusted),
                item.leechers.to_string().fg(theme.remake),
                shorten_number(item.downloads).fg(theme.fg),
            ])
            .aligned(align, binding.to_owned())
            .fg(theme.fg)
        })
        .collect();

    let mut headers = header.get_row(sel_sort.dir, sel_sort.sort as u32);
    if let Some(columns) = columns {
        let cols = columns.array();

        headers.cells = cond_vec!(cols ; headers.cells);
        rows = rows
            .clone()
            .into_iter()
            .map(|mut r| {
                r.cells = cond_vec!(cols ; r.cells.to_owned());
                r
            })
            .collect::<Vec<ResultRow>>();
        binding = cond_vec!(cols ; binding);
    }
    ResultTable {
        headers,
        rows,
        binding,
    }
}

impl Source for NyaaHtmlSource {
    async fn search(
        client: &reqwest::Client,
        search: &SearchQuery,
        config: &SourceConfig,
        date_format: Option<String>,
    ) -> Result<ResultResponse, Box<dyn Error + Send + Sync>> {
        let nyaa = config.nyaa.to_owned().unwrap_or_default();
        if nyaa.rss {
            return nyaa_rss::search_rss(client, search, config, date_format).await;
        }
        let cat = search.category;
        let filter = search.filter;
        let page = search.page;
        let user = search.user.to_owned().unwrap_or_default();
        let sort = NyaaSort::try_from(search.sort.sort)
            .unwrap_or(NyaaSort::Date)
            .to_url();

        let base_url = add_protocol(nyaa.base_url, true);
        // let base_url = add_protocol(ctx.config.base_url.clone(), true);
        let (high, low) = (cat / 10, cat % 10);
        let query = encode(&search.query);
        let dir = search.sort.dir.to_url();
        let url = Url::parse(&base_url)?;
        let mut url_query = url.clone();
        url_query.set_query(Some(&format!(
            "q={}&c={}_{}&f={}&p={}&s={}&o={}&u={}",
            query, high, low, filter, page, sort, dir, user
        )));

        // let client = super::request_client(ctx)?;
        let mut request = client.get(url_query.to_owned());
        if let Some(timeout) = nyaa.timeout {
            request = request.timeout(Duration::from_secs(timeout));
        }
        let response = request.send().await?;
        if response.status() != StatusCode::OK {
            // Throw error if response code is not OK
            let code = response.status().as_u16();
            return Err(format!("{}\nInvalid repsponse code: {}", url_query, code).into());
        }
        let content = response.bytes().await?;
        let doc = Html::parse_document(std::str::from_utf8(&content[..])?);

        // let item_sel = &Selector::parse("table.torrent-list > tbody > tr")?;
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
                let id = torrent
                    .split('/')
                    .last()?
                    .split('.')
                    .next()?
                    .parse::<usize>()
                    .ok()?;
                let id = format!("nyaa-{}", id);
                let file_name = format!("{}.torrent", id);

                let size = inner(e, size_sel, "0 bytes")
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
                    ..Default::default()
                })
            })
            .collect();

        Ok(ResultResponse {
            items,
            total_results,
            last_page,
        })
    }
    async fn sort(
        client: &reqwest::Client,
        search: &SearchQuery,
        config: &SourceConfig,
        date_format: Option<String>,
    ) -> Result<ResultResponse, Box<dyn Error + Send + Sync>> {
        let nyaa = config.nyaa.to_owned().unwrap_or_default();
        let sort = search.sort;
        let mut res = NyaaHtmlSource::search(client, search, config, date_format).await;

        if nyaa.rss {
            if let Ok(res) = &mut res {
                nyaa_rss::sort_items(&mut res.items, sort);
            }
        }
        res
    }
    async fn filter(
        client: &reqwest::Client,
        search: &SearchQuery,
        config: &SourceConfig,
        date_format: Option<String>,
    ) -> Result<ResultResponse, Box<dyn Error + Send + Sync>> {
        NyaaHtmlSource::search(client, search, config, date_format).await
    }
    async fn categorize(
        client: &reqwest::Client,
        search: &SearchQuery,
        config: &SourceConfig,
        date_format: Option<String>,
    ) -> Result<ResultResponse, Box<dyn Error + Send + Sync>> {
        NyaaHtmlSource::search(client, search, config, date_format).await
    }

    fn info() -> SourceInfo {
        let cats = cats! {
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
        };
        SourceInfo {
            cats,
            filters: NyaaFilter::iter().map(|f| f.to_string()).collect(),
            sorts: NyaaSort::iter().map(|item| item.to_string()).collect(),
        }
    }

    fn load_config(config: &mut SourceConfig) {
        if config.nyaa.is_none() {
            config.nyaa = Some(NyaaConfig::default());
        }
    }

    fn default_category(cfg: &SourceConfig) -> usize {
        let default = cfg
            .nyaa
            .as_ref()
            .map(|c| c.default_category.to_owned())
            .unwrap_or_default();
        Self::info().entry_from_cfg(&default).id
    }

    fn default_sort(cfg: &SourceConfig) -> usize {
        cfg.nyaa
            .as_ref()
            .map(|c| c.default_sort as usize)
            .unwrap_or_default()
    }

    fn default_filter(cfg: &SourceConfig) -> usize {
        cfg.nyaa
            .as_ref()
            .map(|c| c.default_filter as usize)
            .unwrap_or_default()
    }

    fn default_search(cfg: &SourceConfig) -> String {
        cfg.nyaa
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
        let nyaa = config.nyaa.to_owned().unwrap_or_default();
        nyaa_table(items, theme, &search.sort, &nyaa.columns)
    }
}
