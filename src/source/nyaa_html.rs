use std::{cmp::max, error::Error, time::Duration};

use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use ratatui::{
    layout::{Alignment, Constraint},
    style::{Color, Stylize as _},
};
use reqwest::StatusCode;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use strum::{Display, FromRepr, VariantArray};
use urlencoding::encode;

use crate::{
    cats, cond_vec,
    results::{ResultColumn, ResultHeader, ResultResponse, ResultRow, ResultTable},
    sel,
    sync::SearchQuery,
    theme::Theme,
    util::{
        conv::{shorten_number, to_bytes},
        html::{as_type, attr, inner},
    },
    widget::sort::{SelectedSort, SortDir},
};

use super::{
    add_protocol, nyaa_rss, Item, ItemType, Source, SourceConfig, SourceInfo, SourceResponse,
};

#[derive(Serialize, Deserialize, Clone, Copy, Default)]
#[serde(default)]
pub struct NyaaTheme {
    #[serde(rename = "categories")]
    cat: NyaaCategoryTheme,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
#[serde(default)]
pub struct NyaaCategoryTheme {
    #[serde(with = "color_to_tui")]
    pub anime_english_translated: Color,
    #[serde(with = "color_to_tui")]
    pub anime_non_english_translated: Color,
    #[serde(with = "color_to_tui")]
    pub anime_raw: Color,
    #[serde(with = "color_to_tui")]
    pub anime_music_video: Color,
    #[serde(with = "color_to_tui")]
    pub audio_lossless: Color,
    #[serde(with = "color_to_tui")]
    pub audio_lossy: Color,
    #[serde(with = "color_to_tui")]
    pub literature_english_translated: Color,
    #[serde(with = "color_to_tui")]
    pub literature_non_english_translated: Color,
    #[serde(with = "color_to_tui")]
    pub literature_raw: Color,
    #[serde(with = "color_to_tui")]
    pub live_english_translated: Color,
    #[serde(with = "color_to_tui")]
    pub live_non_english_translated: Color,
    #[serde(with = "color_to_tui")]
    pub live_idol_promo_video: Color,
    #[serde(with = "color_to_tui")]
    pub live_raw: Color,
    #[serde(with = "color_to_tui")]
    pub picture_graphics: Color,
    #[serde(with = "color_to_tui")]
    pub picture_photos: Color,
    #[serde(with = "color_to_tui")]
    pub software_applications: Color,
    #[serde(with = "color_to_tui")]
    pub software_games: Color,
}

impl Default for NyaaCategoryTheme {
    fn default() -> Self {
        use Color::*;
        Self {
            anime_english_translated: LightMagenta,
            anime_non_english_translated: LightGreen,
            anime_raw: Gray,
            anime_music_video: Magenta,
            audio_lossless: Red,
            audio_lossy: Yellow,
            literature_english_translated: LightGreen,
            literature_non_english_translated: Yellow,
            literature_raw: Gray,
            live_english_translated: Yellow,
            live_non_english_translated: LightCyan,
            live_idol_promo_video: LightYellow,
            live_raw: Gray,
            picture_graphics: LightMagenta,
            picture_photos: Magenta,
            software_applications: Blue,
            software_games: LightBlue,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct NyaaConfig {
    pub base_url: String,
    pub default_sort: NyaaSort,
    pub default_sort_dir: SortDir,
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
            default_sort_dir: SortDir::Desc,
            default_filter: NyaaFilter::NoFilter,
            default_category: "AllCategories".to_owned(),
            default_search: Default::default(),
            rss: false,
            timeout: None,
            columns: None,
        }
    }
}

#[derive(Serialize, Deserialize, Display, Clone, Copy, VariantArray, PartialEq, Eq, FromRepr)]
#[repr(usize)]
pub enum NyaaSort {
    #[strum(serialize = "Date")]
    Date = 0,
    #[strum(serialize = "Downloads")]
    Downloads = 1,
    #[strum(serialize = "Seeders")]
    Seeders = 2,
    #[strum(serialize = "Leechers")]
    Leechers = 3,
    #[strum(serialize = "Size")]
    Size = 4,
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

#[derive(Serialize, Deserialize, Display, Clone, Copy, VariantArray, PartialEq, Eq, FromRepr)]
pub enum NyaaFilter {
    #[allow(clippy::enum_variant_names)]
    #[strum(serialize = "No Filter")]
    NoFilter = 0,
    #[strum(serialize = "No Remakes")]
    NoRemakes = 1,
    #[strum(serialize = "Trusted Only")]
    TrustedOnly = 2,
    #[strum(serialize = "Batches")]
    Batches = 3,
}

pub struct NyaaHtmlSource;

pub fn nyaa_table(
    items: Vec<Item>,
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
        .into_iter()
        .map(|item| {
            ResultRow::new([
                item.icon.label.fg((item.icon.color)(theme)),
                item.title.fg(match item.item_type {
                    ItemType::Trusted => theme.success,
                    ItemType::Remake => theme.error,
                    ItemType::None => theme.fg,
                }),
                item.size.fg(theme.fg),
                item.date.fg(theme.fg),
                item.seeders.to_string().fg(theme.success),
                item.leechers.to_string().fg(theme.error),
                shorten_number(item.downloads).fg(theme.fg),
            ])
            .aligned(align)
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
    ) -> Result<SourceResponse, Box<dyn Error + Send + Sync>> {
        let nyaa = config.nyaa.to_owned().unwrap_or_default();
        if nyaa.rss {
            return nyaa_rss::search_rss::<Self>(
                nyaa.base_url,
                nyaa.timeout,
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

        let base_url = add_protocol(nyaa.base_url, true)?;
        let mut url = base_url.clone();
        // let base_url = add_protocol(ctx.config.base_url.clone(), true);
        let (high, low) = (cat / 10, cat % 10);
        let query = encode(&search.query);
        let dir = search.sort.dir.to_url();
        url.set_query(Some(&format!(
            "q={}&c={}_{}&f={}&p={}&s={}&o={}&u={}",
            query, high, low, filter, page, sort, dir, user
        )));

        let mut request = client.get(url.to_owned());
        if let Some(timeout) = nyaa.timeout {
            request = request.timeout(Duration::from_secs(timeout));
        }
        let response = request.send().await?;
        if response.status() != StatusCode::OK {
            // Throw error if response code is not OK
            let code = response.status().as_u16();
            return Err(format!("{}\nInvalid response code: {}", url, code).into());
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

                let seeders = as_type(inner(e, seed_sel, "0")).unwrap_or_default();
                let leechers = as_type(inner(e, leech_sel, "0")).unwrap_or_default();
                let downloads = as_type(inner(e, dl_sel, "0")).unwrap_or_default();
                let torrent_link = base_url
                    .join(&torrent)
                    .map(Into::into)
                    .unwrap_or("null".to_owned());
                let post_link = base_url
                    .join(&attr(e, title_sel, "href"))
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
            total_results,
            last_page,
        }))
    }
    async fn sort(
        client: &reqwest::Client,
        search: &SearchQuery,
        config: &SourceConfig,
        date_format: Option<String>,
    ) -> Result<SourceResponse, Box<dyn Error + Send + Sync>> {
        let nyaa = config.nyaa.to_owned().unwrap_or_default();
        let sort = search.sort;
        let mut res = NyaaHtmlSource::search(client, search, config, date_format).await;

        if nyaa.rss {
            if let Ok(SourceResponse::Results(res)) = &mut res {
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
    ) -> Result<SourceResponse, Box<dyn Error + Send + Sync>> {
        NyaaHtmlSource::search(client, search, config, date_format).await
    }
    async fn categorize(
        client: &reqwest::Client,
        search: &SearchQuery,
        config: &SourceConfig,
        date_format: Option<String>,
    ) -> Result<SourceResponse, Box<dyn Error + Send + Sync>> {
        NyaaHtmlSource::search(client, search, config, date_format).await
    }
    async fn solve(
        _solution: String,
        client: &reqwest::Client,
        search: &SearchQuery,
        config: &SourceConfig,
        date_format: Option<String>,
    ) -> Result<SourceResponse, Box<dyn Error + Send + Sync>> {
        NyaaHtmlSource::search(client, search, config, date_format).await
    }

    fn info() -> SourceInfo {
        let cats = cats! {
            "All Categories" => {
                0 => ("---", "All Categories", "AllCategories", fg);
            }
            "Anime" => {
                10 => ("Ani", "All Anime", "AllAnime", fg);
                12 => ("Sub", "English Translated", "AnimeEnglishTranslated", source.nyaa.cat.anime_english_translated);
                13 => ("Sub", "Non-English Translated", "AnimeNonEnglishTranslated", source.nyaa.cat.anime_non_english_translated);
                14 => ("Raw", "Raw", "AnimeRaw", source.nyaa.cat.anime_raw);
                11 => ("AMV", "Anime Music Video", "AnimeMusicVideo", source.nyaa.cat.anime_music_video);
            }
            "Audio" => {
                20 => ("Aud", "All Audio", "AllAudio", fg);
                21 => ("Aud", "Lossless", "AudioLossless", source.nyaa.cat.audio_lossless);
                22 => ("Aud", "Lossy", "AudioLossy", source.nyaa.cat.audio_lossy);
            }
            "Literature" => {
                30 => ("Lit", "All Literature", "AllLiterature", fg);
                31 => ("Lit", "English Translated", "LitEnglishTranslated", source.nyaa.cat.literature_english_translated);
                32 => ("Lit", "Non-English Translated", "LitNonEnglishTranslated", source.nyaa.cat.literature_non_english_translated);
                33 => ("Lit", "Raw", "LitRaw", source.nyaa.cat.literature_raw);
            }
            "Live Action" => {
                40 => ("Liv", "All Live Action", "AllLiveAction", fg);
                41 => ("Liv", "English Translated", "LiveEnglishTranslated", source.nyaa.cat.live_english_translated);
                43 => ("Liv", "Non-English Translated", "LiveNonEnglishTranslated", source.nyaa.cat.live_non_english_translated);
                42 => ("Liv", "Idol/Promo Video", "LiveIdolPromoVideo", source.nyaa.cat.live_idol_promo_video);
                44 => ("Liv", "Raw", "LiveRaw", source.nyaa.cat.live_raw);
            }
            "Pictures" => {
                50 => ("Pic", "All Pictures", "AllPictures", fg);
                51 => ("Pic", "Graphics", "PicGraphics", source.nyaa.cat.picture_graphics);
                52 => ("Pic", "Photos", "PicPhotos", source.nyaa.cat.picture_photos);
            }
            "Software" => {
                60 => ("Sof", "All Software", "AllSoftware", fg);
                61 => ("Sof", "Applications", "SoftApplications", source.nyaa.cat.software_applications);
                62 => ("Sof", "Games", "SoftGames", source.nyaa.cat.software_games);
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

    fn default_sort(cfg: &SourceConfig) -> SelectedSort {
        cfg.nyaa
            .as_ref()
            .map(|c| SelectedSort {
                sort: c.default_sort as usize,
                dir: c.default_sort_dir,
            })
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
        nyaa_table(items.into(), theme, &search.sort, &nyaa.columns)
    }
}
