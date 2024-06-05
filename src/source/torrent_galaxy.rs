use std::{
    cmp::max,
    collections::HashMap,
    error::Error,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use ratatui::{
    layout::{Alignment, Constraint},
    style::{Color, Stylize},
};
use reqwest::{StatusCode, Url};
use scraper::{selectable::Selectable, Html, Selector};
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
        html::{as_type, attr, inner},
    },
    widget::EnumIter as _,
};

use super::{add_protocol, Item, ItemType, Source, SourceConfig, SourceInfo, SourceResponse};

#[derive(Serialize, Deserialize, Clone, Copy)]
#[serde(default)]
pub struct TgxTheme {
    #[serde(with = "color_to_tui")]
    pub all_categories: Color,
    #[serde(with = "color_to_tui")]
    pub movies_4k: Color,
    #[serde(with = "color_to_tui")]
    pub movies_bollywood: Color,
    #[serde(with = "color_to_tui")]
    pub movies_cam: Color,
    #[serde(with = "color_to_tui")]
    pub movies_hd: Color,
    #[serde(with = "color_to_tui")]
    pub movies_pack: Color,
    #[serde(with = "color_to_tui")]
    pub movies_sd: Color,
    #[serde(with = "color_to_tui")]
    pub tv_hd: Color,
    #[serde(with = "color_to_tui")]
    pub tv_sd: Color,
    #[serde(with = "color_to_tui")]
    pub tv_4k: Color,
    #[serde(with = "color_to_tui")]
    pub tv_pack: Color,
    #[serde(with = "color_to_tui")]
    pub tv_sports: Color,
    #[serde(with = "color_to_tui")]
    pub anime: Color,
    #[serde(with = "color_to_tui")]
    pub apps_mobile: Color,
    #[serde(with = "color_to_tui")]
    pub apps_other: Color,
    #[serde(with = "color_to_tui")]
    pub apps_windows: Color,
    #[serde(with = "color_to_tui")]
    pub audiobooks: Color,
    #[serde(with = "color_to_tui")]
    pub comics: Color,
    #[serde(with = "color_to_tui")]
    pub ebooks: Color,
    #[serde(with = "color_to_tui")]
    pub educational: Color,
    #[serde(with = "color_to_tui")]
    pub magazines: Color,
    #[serde(with = "color_to_tui")]
    pub documentaries: Color,
    #[serde(with = "color_to_tui")]
    pub games_windows: Color,
    #[serde(with = "color_to_tui")]
    pub games_other: Color,
    #[serde(with = "color_to_tui")]
    pub music_albums: Color,
    #[serde(with = "color_to_tui")]
    pub music_discography: Color,
    #[serde(with = "color_to_tui")]
    pub music_lossless: Color,
    #[serde(with = "color_to_tui")]
    pub music_video: Color,
    #[serde(with = "color_to_tui")]
    pub music_singles: Color,
    #[serde(with = "color_to_tui")]
    pub audio_other: Color,
    #[serde(with = "color_to_tui")]
    pub pictures_other: Color,
    #[serde(with = "color_to_tui")]
    pub training_other: Color,
    #[serde(with = "color_to_tui")]
    pub other: Color,
    #[serde(with = "color_to_tui")]
    pub xxx_4k: Color,
    #[serde(with = "color_to_tui")]
    pub xxx_hd: Color,
    #[serde(with = "color_to_tui")]
    pub xxx_misc: Color,
    #[serde(with = "color_to_tui")]
    pub xxx_sd: Color,
}

impl Default for TgxTheme {
    fn default() -> Self {
        use Color::*;
        Self {
            all_categories: White,
            movies_4k: LightMagenta,
            movies_bollywood: Green,
            movies_cam: LightCyan,
            movies_hd: LightBlue,
            movies_pack: Magenta,
            movies_sd: Yellow,
            tv_hd: Green,
            tv_sd: LightCyan,
            tv_4k: LightMagenta,
            tv_pack: Blue,
            tv_sports: LightGreen,
            anime: LightMagenta,
            apps_mobile: LightGreen,
            apps_other: Magenta,
            apps_windows: LightCyan,
            audiobooks: Yellow,
            comics: LightGreen,
            ebooks: Green,
            educational: Yellow,
            magazines: Green,
            documentaries: LightYellow,
            games_windows: LightCyan,
            games_other: Yellow,
            music_albums: Cyan,
            music_discography: Magenta,
            music_lossless: LightBlue,
            music_video: Green,
            music_singles: LightYellow,
            audio_other: LightGreen,
            pictures_other: Green,
            training_other: LightBlue,
            other: Yellow,
            xxx_4k: Red,
            xxx_hd: Red,
            xxx_misc: Red,
            xxx_sd: Red,
        }
    }
}

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
    imdb: Option<bool>,
    uploader: Option<bool>,
    size: Option<bool>,
    date: Option<bool>,
    seeders: Option<bool>,
    leechers: Option<bool>,
    views: Option<bool>,
}

impl TgxColumns {
    fn array(self) -> [bool; 10] {
        [
            self.category.unwrap_or(true),
            self.language.unwrap_or(true),
            self.title.unwrap_or(true),
            self.imdb.unwrap_or(true),
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

fn get_url(
    base_url: String,
    search: &SearchQuery,
) -> Result<(Url, Url), Box<dyn Error + Send + Sync>> {
    let base_url = Url::parse(&add_protocol(base_url, true))?.join("torrents.php")?;

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
        Ok(TgxFilter::ExcludeXXX) => "&nox=1",
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
    Ok((base_url, url))
}

async fn try_get_content(
    client: &reqwest::Client,
    timeout: Option<u64>,
    url: &Url,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let mut request = client.get(url.to_owned());
    if let Some(timeout) = timeout {
        request = request.timeout(Duration::from_secs(timeout));
    }
    let response = request
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:126.0) Gecko/20100101 Firefox/126.0",
        )
        .send()
        .await?;
    if response.status() != StatusCode::OK {
        // Throw error if response code is not OK
        let code = response.status().as_u16();
        return Err(format!("{}\nInvalid response code: {}", url, code).into());
    }
    Ok(response.text().await?)
}

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
    ) -> Result<SourceResponse, Box<dyn Error + Send + Sync>> {
        TorrentGalaxyHtmlSource::search(client, search, config, date_format).await
    }
    async fn categorize(
        client: &reqwest::Client,
        search: &SearchQuery,
        config: &SourceConfig,
        date_format: Option<String>,
    ) -> Result<SourceResponse, Box<dyn Error + Send + Sync>> {
        TorrentGalaxyHtmlSource::search(client, search, config, date_format).await
    }
    async fn sort(
        client: &reqwest::Client,
        search: &SearchQuery,
        config: &SourceConfig,
        date_format: Option<String>,
    ) -> Result<SourceResponse, Box<dyn Error + Send + Sync>> {
        TorrentGalaxyHtmlSource::search(client, search, config, date_format).await
    }

    async fn search(
        client: &reqwest::Client,
        search: &SearchQuery,
        config: &SourceConfig,
        _date_format: Option<String>,
    ) -> Result<SourceResponse, Box<dyn Error + Send + Sync>> {
        let tgx = config.tgx.to_owned().unwrap_or_default();
        let (base_url, url) = get_url(tgx.base_url, search)?;

        let table_sel = &sel!(".tgxtable")?;

        // If that doesn't work, try making the user solve a captcha
        let content = try_get_content(client, tgx.timeout, &url).await?;
        if Html::parse_document(&content).select(table_sel).count() == 0 {
            let mut request = client.get("https://torrentgalaxy.to/captcha/cpt_show.pnp?v=txlight&63fd4c746843c74b53ca60277192fb48");
            if let Some(timeout) = tgx.timeout {
                request = request.timeout(Duration::from_secs(timeout));
            }
            let response = request
                .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:126.0) Gecko/20100101 Firefox/126.0")
                .send()
                .await?;
            let bytes = response.bytes().await?;
            let mut picker = ratatui_image::picker::Picker::new((1, 2));
            picker.protocol_type = ratatui_image::picker::ProtocolType::Halfblocks;
            let dyn_image = image::load_from_memory(&bytes[..])?;
            let image = picker.new_resize_protocol(dyn_image);

            return Ok(SourceResponse::Captcha(image));
        }

        // Results table found, can start parsing
        let doc = Html::parse_document(&content);

        let item_sel = &sel!("div.tgxtablerow")?;
        let title_sel = &sel!("div.tgxtablecell:nth-of-type(4) > div > a.txlight")?;
        let imdb_sel = &sel!("div.tgxtablecell:nth-of-type(4) > div > a:last-of-type")?;
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
                let seeders = as_type(inner(e, seed_sel, "0")).unwrap_or_default();
                let leechers = as_type(inner(e, leech_sel, "0")).unwrap_or_default();
                let views = as_type(inner(e, views_sel, "0")).unwrap_or_default();
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

                let torrent_link = base_url
                    .join(&attr(e, torrent_sel, "href"))
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

                let imdb = attr(e, imdb_sel, "href");
                let imdb = match imdb.rsplit_once('=').map(|r| r.1).unwrap_or("") {
                    "tt2000000" => "", // For some reason, most XXX titles use this ID
                    i => i,
                };

                let extra: HashMap<String, String> = collection![
                    "uploader".to_owned() => inner(e, uploader_sel, "???"),
                    "uploader_status".to_owned() => attr(e, uploader_status_sel, "title"),
                    "lang".to_owned() => attr(e, lang_sel, "title"),
                    "imdb".to_owned() => imdb.to_owned(),
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

        Ok(SourceResponse::Results(ResultResponse {
            items,
            total_results,
            last_page,
        }))
    }

    async fn solve(
        solution: String,
        client: &reqwest::Client,
        search: &SearchQuery,
        config: &SourceConfig,
        date_format: Option<String>,
    ) -> Result<SourceResponse, Box<dyn Error + Send + Sync>> {
        let tgx = config.tgx.to_owned().unwrap_or_default();
        let time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();

        let hash = "4578678889c4b42ae37b543434c81d85";
        // let hash = "ff9df5a6db0ebe6bd636296da767a587";
        let base_url = Url::parse(&tgx.base_url)?;
        let mut hash_url = base_url.clone().join("hub.php")?;
        hash_url.set_query(Some(&format!("a=vlad&u={}", time)));
        // let hash_url = format!("https://torrentgalaxy.to/hub.php?a=vlad&u={}", time);
        client
            .post(hash_url.clone())
            .body(format!("fash={}", hash))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header(
                "User-Agent",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:126.0) Gecko/20100101 Firefox/126.0",
            )
            .send()
            .await?;

        let (_base_url, url) = get_url(tgx.base_url, search)?;
        let mut full_url = base_url.clone().join("galaxyfence.php")?;
        full_url.set_query(Some(&format!(
            "captcha={}&dropoff={}",
            solution,
            encode(&format!(
                "{}?{}",
                url.path(),
                url.query().unwrap_or_default()
            ))
        )));
        let mut request = client.post(full_url.clone());
        if let Some(timeout) = tgx.timeout {
            request = request.timeout(Duration::from_secs(timeout));
        }
        request = request.header(
            "Accept",
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8",
        )
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:126.0) Gecko/20100101 Firefox/126.0")
            .header("Content-Type", "application/x-www-form-urlencoded");

        let response = request.send().await?;
        if response.status() != StatusCode::OK {
            return Err(format!(
                "Captcha solution returned HTTP status {}",
                response.status()
            )
            .into());
        }

        TorrentGalaxyHtmlSource::search(client, search, config, date_format).await
    }

    fn info() -> SourceInfo {
        let cats = cats! {
            "All Categories" => { 0 => ("---", "All Categories", "AllCategories", tgx.all_categories); }
            "Movies" => {3 => ("4kM", "4K UHD Movies", "4kMovies", tgx.movies_4k);
                46 => ("Bly", "Bollywood", "Bollywood Movies", tgx.movies_bollywood);
                45 => ("Cam", "Cam/TS", "CamMovies", tgx.movies_cam);
                42 => ("HdM", "HD Movies", "HdMovies", tgx.movies_hd);
                4 => ("PkM", "Movie Packs", "PackMovies", tgx.movies_pack);
                1 => ("SdM", "SD Movies", "SdMovies", tgx.movies_sd);}
            "TV" => {41 => ("HdT", "TV HD", "HdTV", tgx.tv_hd);
                5 => ("SdT", "TV SD", "SdTV", tgx.tv_sd);
                11 => ("4kT", "TV 4k", "4kTV", tgx.tv_4k);
                6 => ("PkT", "TV Packs", "PacksTV", tgx.tv_pack);
                7 => ("Spo", "Sports", "SportsTV", tgx.tv_sports);}
            "Anime" => {28 => ("Ani", "All Anime", "Anime", tgx.anime);}
            "Apps" => {20 => ("Mob", "Mobile Apps", "AppsMobile", tgx.apps_mobile);
                21 => ("App", "Other Apps", "AppsOther", tgx.apps_other);
                18 => ("Win", "Windows Apps", "AppsWindows", tgx.apps_windows);}
            "Books" => {13 => ("Abk", "Audiobooks", "Audiobooks", tgx.audiobooks);
                19 => ("Com", "Comics", "Comics", tgx.comics);
                12 => ("Ebk", "Ebooks", "Ebooks", tgx.ebooks);
                14 => ("Edu", "Educational", "Educational", tgx.educational);
                15 => ("Mag", "Magazines", "Magazines", tgx.magazines);}
            "Documentaries" => {9 => ("Doc", "All Documentaries", "Documentaries", tgx.documentaries);}
            "Games" => {10 => ("Wgm", "Windows Games", "WindowsGames", tgx.games_windows);
                43 => ("Ogm", "Other Games", "OtherGames", tgx.games_other);}
            "Music" => {22 => ("Alb", "Music Albums", "AlbumsMusic", tgx.music_albums);
                26 => ("Dis", "Music Discography", "DiscographyMusic", tgx.music_discography);
                23 => ("Los", "Music Lossless", "LosslessMusic", tgx.music_lossless);
                25 => ("MV ", "Music Video", "MusicVideo", tgx.music_video);
                24 => ("Sin", "Music Singles", "SinglesMusic", tgx.music_singles);}
            "Other" => {17 => ("Aud", "Other Audio", "AudioOther", tgx.audio_other);
                40 => ("Pic", "Other Pictures", "PicturesOther", tgx.pictures_other);
                37 => ("Tra", "Other Training", "TrainingOther", tgx.training_other);
                33 => ("Oth", "Other", "Other", tgx.other);}
            "XXX" => {48 => ("4kX", "XXX 4k", "4kXXX", tgx.xxx_4k);
                35 => ("HdX", "XXX HD", "HdXXX", tgx.xxx_hd);
                47 => ("MsX", "XXX Misc", "MiscXXX", tgx.xxx_misc);
                34 => ("SdX", "XXX SD", "SdXXX", tgx.xxx_sd);}
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
        let raw_imdb_width = items
            .iter()
            .map(|i| i.extra.get("imdb").map(|u| u.len()).unwrap_or(0))
            .max()
            .unwrap_or_default() as u16;
        let imdb_width = max(raw_imdb_width, 4);

        let header = ResultHeader::new([
            ResultColumn::Normal("Cat".to_owned(), Constraint::Length(3)),
            ResultColumn::Normal("".to_owned(), Constraint::Length(2)),
            ResultColumn::Normal("Name".to_owned(), Constraint::Min(3)),
            ResultColumn::Normal("imdb".to_owned(), Constraint::Length(imdb_width)),
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
                    item.icon.label.fg((item.icon.color)(theme)),
                    item.extra
                        .get("lang")
                        .map(|l| get_lang(l.to_owned()))
                        .unwrap_or("??".to_owned())
                        .fg(theme.fg),
                    item.title.to_owned().fg(match item.item_type {
                        ItemType::Trusted => theme.success,
                        ItemType::Remake => theme.error,
                        ItemType::None => theme.fg,
                    }),
                    item.extra
                        .get("imdb")
                        .cloned()
                        .unwrap_or_default()
                        .fg(theme.fg),
                    item.extra
                        .get("uploader")
                        .cloned()
                        .unwrap_or("???".to_owned())
                        .fg(item
                            .extra
                            .get("uploader_status")
                            .and_then(|u| get_status_color(u.to_owned()))
                            .unwrap_or(theme.fg)),
                    item.size.clone().fg(theme.fg),
                    item.date.clone().fg(theme.fg),
                    item.seeders.to_string().fg(theme.success),
                    item.leechers.to_string().fg(theme.error),
                    shorten_number(item.downloads).fg(theme.fg),
                ])
                .aligned(align)
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
