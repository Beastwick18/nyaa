use std::{cmp::max, collections::HashMap, error::Error, time::Duration};

use ratatui::{
    layout::{Alignment, Constraint},
    style::{Color, Stylize},
};
use reqwest::{StatusCode, Url};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use urlencoding::encode;

use crate::{
    cats, collection, cond_vec, popup_enum,
    results::{ResultColumn, ResultHeader, ResultResponse, ResultRow, ResultTable},
    sel,
    sync::SearchQuery,
    theme::Theme,
    util::{
        conv::{shorten_number, to_bytes},
        html::{attr, inner},
    },
    widget::EnumIter as _,
};

use super::{add_protocol, Item, ItemType, Source, SourceConfig, SourceInfo};

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct TgxConfig {
    pub base_url: String,
    pub default_sort: TgxSort,
    pub default_filter: TgxFilter,
    pub default_category: String,
    pub default_search: String,
    pub timeout: Option<u64>,
    pub columns: Option<TgxColumns>,
}

impl Default for TgxConfig {
    fn default() -> Self {
        Self {
            base_url: "https://torrentgalaxy.to/".to_owned(),
            default_sort: TgxSort::Date,
            default_filter: TgxFilter::NoFilter,
            default_category: "AllCategories".to_owned(),
            default_search: Default::default(),
            timeout: None,
            columns: None,
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Default)]
pub struct TgxColumns {
    category: Option<bool>,
    language: Option<bool>,
    title: Option<bool>,
    uploader: Option<bool>,
    size: Option<bool>,
    date: Option<bool>,
    seeders: Option<bool>,
    leechers: Option<bool>,
    views: Option<bool>,
}

impl TgxColumns {
    fn array(self) -> [bool; 9] {
        [
            self.category.unwrap_or(true),
            self.language.unwrap_or(true),
            self.title.unwrap_or(true),
            self.uploader.unwrap_or(true),
            self.size.unwrap_or(true),
            self.date.unwrap_or(true),
            self.seeders.unwrap_or(true),
            self.leechers.unwrap_or(true),
            self.views.unwrap_or(true),
        ]
    }
}

popup_enum! {
    TgxSort;
    (0, Date, "Date");
    (1, Seeders, "Seeders");
    (2, Leechers, "Leechers");
    (3, Size, "Size");
    (4, Name, "Name");
}

popup_enum! {
    TgxFilter;
    #[allow(clippy::enum_variant_names)]
    (0, NoFilter, "NoFilter");
    (1, OnlineStreams, "Filter online streams");
    (2, ExcludeXXX, "Exclude XXX");
    (3, NoWildcard, "No wildcard");
}

pub struct TorrentGalaxyHtmlSource;

fn get_lang(full_name: String) -> String {
    match full_name.as_str() {
        "English" => "en",
        "French" => "fr",
        "German" => "de",
        "Italian" => "it",
        "Japanese" => "jp",
        "Spanish" => "es",
        "Russian" => "ru",
        "Norwegian" => "no",
        "Hindi" => "hi",
        "Korean" => "ko",
        "Danish" => "da",
        "Dutch" => "nl",
        "Chinese" => "zh",
        "Portuguese" => "pt",
        "Polish" => "pl",
        "Turkish" => "tr",
        "Telugu" => "te",
        "Swedish" => "sv",
        "Czech" => "cs",
        "Arabic" => "ar",
        "Romanian" => "ro",
        "Bengali" => "bn",
        "Urdu" => "ur",
        "Thai" => "th",
        "Tamil" => "ta",
        "Croatian" => "hr",
        _ => "??",
    }
    .to_owned()
}

fn get_status_color(status: String) -> Option<Color> {
    match status.as_str() {
        "Trial Uploader" => Some(Color::Magenta),
        "Trusted Uploader" => Some(Color::LightGreen),
        "Trusted User" => Some(Color::Cyan),
        "Moderator" => Some(Color::Red),
        "Admin" => Some(Color::Yellow),
        "Torrent Officer" => Some(Color::LightYellow),
        "Verified Uploader" => Some(Color::LightBlue),
        _ => None,
    }
}

impl Source for TorrentGalaxyHtmlSource {
    async fn filter(
        client: &reqwest::Client,
        search: &SearchQuery,
        config: &SourceConfig,
        date_format: Option<String>,
    ) -> Result<ResultResponse, Box<dyn Error + Send + Sync>> {
        TorrentGalaxyHtmlSource::search(client, search, config, date_format).await
    }
    async fn categorize(
        client: &reqwest::Client,
        search: &SearchQuery,
        config: &SourceConfig,
        date_format: Option<String>,
    ) -> Result<ResultResponse, Box<dyn Error + Send + Sync>> {
        TorrentGalaxyHtmlSource::search(client, search, config, date_format).await
    }
    async fn sort(
        client: &reqwest::Client,
        search: &SearchQuery,
        config: &SourceConfig,
        date_format: Option<String>,
    ) -> Result<ResultResponse, Box<dyn Error + Send + Sync>> {
        TorrentGalaxyHtmlSource::search(client, search, config, date_format).await
    }
    async fn search(
        client: &reqwest::Client,
        search: &SearchQuery,
        config: &SourceConfig,
        _date_format: Option<String>,
    ) -> Result<ResultResponse, Box<dyn Error + Send + Sync>> {
        let tgx = config.tgx.to_owned().unwrap_or_default();
        let base_url = Url::parse(&add_protocol(tgx.base_url, true))?.join("torrents.php")?;
        let query = encode(&search.query);

        let sort = match TgxSort::try_from(search.sort.sort) {
            Ok(TgxSort::Date) => "&sort=id",
            Ok(TgxSort::Seeders) => "&sort=seeders",
            Ok(TgxSort::Leechers) => "&sort=leechers",
            Ok(TgxSort::Size) => "&sort=size",
            Ok(TgxSort::Name) => "&sort=name",
            _ => "",
        };
        let ord = format!("&order={}", search.sort.dir.to_url());
        let filter = match TgxFilter::try_from(search.filter) {
            Ok(TgxFilter::OnlineStreams) => "&filterstream=1",
            Ok(TgxFilter::ExcludeXXX) => "&nox=2&nox=1",
            Ok(TgxFilter::NoWildcard) => "&nowildcard=1",
            _ => "",
        };
        let cat = match search.category {
            0 => "".to_owned(),
            x => format!("&c{}=1", x),
        };

        let q = format!(
            "search={}&page={}{}{}{}{}",
            query,
            search.page - 1,
            filter,
            cat,
            sort,
            ord
        );
        let mut url = base_url.clone();
        url.set_query(Some(&q));

        let mut request = client.get(url.to_owned());
        if let Some(timeout) = tgx.timeout {
            request = request.timeout(Duration::from_secs(timeout));
        }
        let response = request.send().await?;
        if response.status() != StatusCode::OK {
            // Throw error if response code is not OK
            let code = response.status().as_u16();
            return Err(format!("{}\nInvalid response code: {}", url, code).into());
        }
        let content = response.text().await?;
        let doc = Html::parse_document(&content);

        let table_sel = &sel!(".tgxtable")?;
        if doc.select(table_sel).count() == 0 {
            return Err(format!(
                "{}\nNo results table found:\nMost likely due to captcha or rate limit\n\nWait a bit before searching again...",
                url,
            )
            .into());
        }
        let item_sel = &sel!("div.tgxtablerow")?;
        let title_sel = &sel!("div.tgxtablecell:nth-of-type(4) > div > a.txlight")?;
        let cat_sel = &sel!("div.tgxtablecell:nth-of-type(1) > a")?;
        let date_sel = &sel!("div.tgxtablecell:nth-of-type(12)")?;
        let seed_sel = &sel!("div.tgxtablecell:nth-of-type(11) > span > font:first-of-type > b")?;
        let leech_sel = &sel!("div.tgxtablecell:nth-of-type(11) > span > font:last-of-type > b")?;
        let size_sel = &sel!("div.tgxtablecell:nth-of-type(8) > span")?;
        let trust_sel = &sel!("div.tgxtablecell:nth-of-type(2) > i")?;
        let views_sel = &sel!("div.tgxtablecell:nth-of-type(10) > span > font > b")?;
        let torrent_sel = &sel!("div.tgxtablecell:nth-of-type(5) > a:first-of-type")?;
        let magnet_sel = &sel!("div.tgxtablecell:nth-of-type(5) > a:last-of-type")?;
        let lang_sel = &sel!("div.tgxtablecell:nth-of-type(3) > img")?;
        let uploader_sel = &sel!("div.tgxtablecell:nth-of-type(7) > span > a > span")?;
        let uploader_status_sel = &sel!("div.tgxtablecell:nth-of-type(7) > span > a")?;

        let pagination_sel = &sel!("div#filterbox2 > span.badge")?;

        let items = doc
            .select(item_sel)
            .filter_map(|e| {
                let cat_id = attr(e, cat_sel, "href")
                    .rsplit_once('=')
                    .map(|v| v.1)
                    .and_then(|v| v.parse::<usize>().ok())
                    .unwrap_or_default();
                let icon = Self::info().entry_from_id(cat_id).icon;
                let date = e
                    .select(date_sel)
                    .nth(0)
                    .map(|e| e.text().collect())
                    .unwrap_or_default();
                let seeders = inner(e, seed_sel, "0")
                    .chars()
                    .filter(char::is_ascii_digit)
                    .collect::<String>()
                    .parse::<u32>()
                    .unwrap_or_default();
                let leechers = inner(e, leech_sel, "0")
                    .chars()
                    .filter(char::is_ascii_digit)
                    .collect::<String>()
                    .parse::<u32>()
                    .unwrap_or_default();
                let views = inner(e, views_sel, "0")
                    .chars()
                    .filter(char::is_ascii_digit)
                    .collect::<String>()
                    .parse::<u32>()
                    .unwrap_or_default();
                let mut size = inner(e, size_sel, "0 MB");

                // Convert numbers like 1,015 KB => 1.01 MB
                if let Some((x, y)) = size.split_once(',') {
                    if let Some((y, unit)) = y.split_once(' ') {
                        let y = y.get(0..2).unwrap_or("00");
                        // find next unit up
                        let unit = match unit.to_lowercase().as_str() {
                            "b" => "kB",
                            "kb" => "MB",
                            "mb" => "GB",
                            "gb" => "TB",
                            _ => "??",
                        };
                        size = format!("{}.{} {}", x, y, unit);
                    }
                }

                let item_type = match e
                    .select(trust_sel)
                    .nth(0)
                    .map(|v| v.value().classes().any(|e| e == "fa-check"))
                    .unwrap_or(false)
                {
                    true => ItemType::None,
                    false => ItemType::Remake,
                };

                let torrent_link = attr(e, torrent_sel, "href");
                let torrent_link = base_url
                    .join(&torrent_link)
                    .map(|u| u.to_string())
                    .unwrap_or_default();
                let magnet_link = attr(e, magnet_sel, "href");
                let post_link = attr(e, title_sel, "href");

                let binding = post_link.split('/').collect::<Vec<&str>>();
                let id = format!("tgx-{}", binding.get(2)?);

                let post_link = base_url
                    .join(&post_link)
                    .map(|u| u.to_string())
                    .unwrap_or_default();
                let hash = torrent_link.split('/').nth(4).unwrap_or("unknown");
                let file_name = format!("{}.torrent", hash);

                let extra: HashMap<String, String> = collection![
                    "uploader".to_owned() => inner(e, uploader_sel, "???"),
                    "uploader_status".to_owned() => attr(e, uploader_status_sel, "title"),
                    "lang".to_owned() => attr(e, lang_sel, "title"),
                ];

                Some(Item {
                    id,
                    date,
                    seeders,
                    leechers,
                    downloads: views,
                    bytes: to_bytes(&size),
                    size,
                    title: attr(e, title_sel, "title"),
                    torrent_link,
                    magnet_link,
                    post_link,
                    file_name,
                    category: cat_id,
                    icon,
                    item_type,
                    extra,
                })
            })
            .collect::<Vec<Item>>();

        let mut last_page = 50;
        let mut total_results = 2500;
        if let Some(pagination) = doc.select(pagination_sel).nth(0) {
            if let Ok(num_results) = pagination
                .inner_html()
                .chars()
                .filter(|c| c.is_ascii_digit())
                .collect::<String>()
                .parse::<usize>()
            {
                if num_results != 0 || items.is_empty() {
                    last_page = (num_results + 49) / 50;
                    total_results = num_results;
                }
            }
        }

        Ok(ResultResponse {
            items,
            total_results,
            last_page,
        })
    }

    fn info() -> SourceInfo {
        let cats = cats! {
            "All Categories" => {
                0 => ("---", "All Categories", "AllCategories", White);
            }
            "Movies" => {
                3 => ("4kM", "4K UHD Movies", "4kMovies", LightMagenta);
                46 => ("Bly", "Bollywood", "Bollywood Movies", Green);
                45 => ("Cam", "Cam/TS", "CamMovies", LightCyan);
                42 => ("HdM", "HD Movies", "HdMovies", LightBlue);
                4 => ("PkM", "Movie Packs", "PackMovies", Magenta);
                1 => ("SdM", "SD Movies", "SdMovies", Yellow);
            }
            "TV" => {
                41 => ("HdT", "TV HD", "HdTV", Green);
                5 => ("SdT", "TV SD", "SdTV", LightCyan);
                11 => ("4kT", "TV 4k", "4kTV", LightMagenta);
                6 => ("PkT", "TV Packs", "PacksTV", Blue);
                7 => ("Spo", "Sports", "SportsTV", LightGreen);
            }
            "Anime" => {
                28 => ("Ani", "All Anime", "Anime", LightMagenta);
            }
            "Apps" => {
                20 => ("Mob", "Mobile Apps", "AppsMobile", LightGreen);
                21 => ("App", "Other Apps", "AppsOther", Magenta);
                18 => ("Win", "Windows Apps", "AppsWindows", LightCyan);
            }
            "Books" => {
                13 => ("Abk", "Audiobooks", "Audiobooks", Yellow);
                19 => ("Com", "Comics", "Comics", LightGreen);
                12 => ("Ebk", "Ebooks", "Ebooks", Green);
                14 => ("Edu", "Educational", "Educational", Yellow);
                15 => ("Mag", "Magazines", "Magazines", Green);
            }
            "Documentaries" => {
                9 => ("Doc", "All Documentaries", "Documentaries", LightYellow);
            }
            "Games" => {
                10 => ("Wgm", "Windows Games", "WindowsGames", LightCyan);
                43 => ("Ogm", "Other Games", "OtherGames", Yellow);
            }
            "Music" => {
                22 => ("Alb", "Music Albums", "AlbumsMusic", Cyan);
                26 => ("Dis", "Music Discography", "DiscographyMusic", Magenta);
                23 => ("Los", "Music Lossless", "LosslessMusic", LightBlue);
                25 => ("MV ", "Music Video", "MusicVideo", Green);
                24 => ("Sin", "Music Singles", "SinglesMusic", LightYellow);
            }
            "Other" => {
                17 => ("Aud", "Other Audio", "AudioOther", LightGreen);
                40 => ("Pic", "Other Pictures", "PicturesOther", Green);
                37 => ("Tra", "Other Training", "TrainingOther", LightBlue);
                33 => ("Oth", "Other", "Other", Yellow);
            }
            "XXX" => {
                48 => ("4kX", "XXX 4k", "4kXXX", Red);
                35 => ("HdX", "XXX HD", "HdXXX", Red);
                47 => ("MsX", "XXX Misc", "MiscXXX", Red);
                34 => ("SdX", "XXX SD", "SdXXX", Red);
            }
        };
        SourceInfo {
            cats,
            filters: TgxFilter::iter().map(|f| f.to_string()).collect(),
            sorts: TgxSort::iter().map(|item| item.to_string()).collect(),
        }
    }

    fn load_config(config: &mut SourceConfig) {
        if config.tgx.is_none() {
            config.tgx = Some(TgxConfig::default());
        }
    }

    fn default_category(cfg: &SourceConfig) -> usize {
        let default = cfg
            .tgx
            .as_ref()
            .map(|c| c.default_category.to_owned())
            .unwrap_or_default();
        Self::info().entry_from_cfg(&default).id
    }

    fn default_sort(cfg: &SourceConfig) -> usize {
        cfg.tgx
            .as_ref()
            .map(|c| c.default_sort as usize)
            .unwrap_or_default()
    }

    fn default_filter(cfg: &SourceConfig) -> usize {
        cfg.tgx
            .as_ref()
            .map(|c| c.default_filter as usize)
            .unwrap_or_default()
    }

    fn default_search(cfg: &SourceConfig) -> String {
        cfg.tgx
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
        let tgx = config.tgx.to_owned().unwrap_or_default();
        let raw_date_width = items.iter().map(|i| i.date.len()).max().unwrap_or_default() as u16;
        let date_width = max(raw_date_width, 6);

        let raw_uploader_width = items
            .iter()
            .map(|i| i.extra.get("uploader").map(|u| u.len()).unwrap_or(0))
            .max()
            .unwrap_or_default() as u16;
        let uploader_width = max(raw_uploader_width, 8);

        let header = ResultHeader::new([
            ResultColumn::Normal("Cat".to_owned(), Constraint::Length(3)),
            ResultColumn::Normal("".to_owned(), Constraint::Length(2)),
            ResultColumn::Normal("Name".to_owned(), Constraint::Min(3)),
            ResultColumn::Normal("Uploader".to_owned(), Constraint::Length(uploader_width)),
            ResultColumn::Sorted("Size".to_owned(), 9, TgxSort::Size as u32),
            ResultColumn::Sorted("Date".to_owned(), date_width, TgxSort::Date as u32),
            ResultColumn::Sorted("".to_owned(), 4, TgxSort::Seeders as u32),
            ResultColumn::Sorted("".to_owned(), 4, TgxSort::Leechers as u32),
            ResultColumn::Normal("  󰈈".to_owned(), Constraint::Length(5)),
        ]);
        let mut binding = header.get_binding();
        let align = [
            Alignment::Left,
            Alignment::Left,
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
                    item.extra
                        .get("lang")
                        .map(|l| get_lang(l.to_owned()))
                        .unwrap_or("??".to_owned())
                        .fg(theme.fg),
                    item.title.to_owned().fg(match item.item_type {
                        ItemType::Trusted => theme.trusted,
                        ItemType::Remake => theme.remake,
                        ItemType::None => theme.fg,
                    }),
                    item.extra
                        .get("uploader")
                        .unwrap_or(&"???".to_owned())
                        .to_owned()
                        .fg(item
                            .extra
                            .get("uploader_status")
                            .and_then(|u| get_status_color(u.to_owned()))
                            .unwrap_or(theme.fg)),
                    item.size.clone().fg(theme.fg),
                    item.date.clone().fg(theme.fg),
                    item.seeders.to_string().fg(theme.trusted),
                    item.leechers.to_string().fg(theme.remake),
                    shorten_number(item.downloads).fg(theme.fg),
                ])
                .aligned(align, &binding)
                .fg(theme.fg)
            })
            .collect();
        let mut headers = header.get_row(search.sort.dir, search.sort.sort as u32);
        if let Some(columns) = tgx.columns {
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
}
