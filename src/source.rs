use std::error::Error;

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{
    app::{App, LoadType, Widgets},
    widget::{category::CatIcon, EnumIter},
};

use self::{nyaa_html::NyaaHtmlSource, nyaa_rss::NyaaRssSource};

pub mod nyaa_html;
pub mod nyaa_rss;

pub fn add_protocol<S: Into<String>>(url: S, default_https: bool) -> String {
    let protocol = match default_https {
        true => "https",
        false => "http",
    };
    let url = url.into();
    let re = Regex::new(r"^https?://.+$").unwrap();
    match re.is_match(&url) {
        true => url,
        // Assume http(s) if not present
        false => format!("{}://{}", protocol, url),
    }
}

#[derive(Clone)]
pub struct Item {
    pub index: usize,
    pub date: String,
    pub seeders: u32,
    pub leechers: u32,
    pub downloads: u32,
    pub size: String,
    pub bytes: usize,
    pub title: String,
    pub torrent_link: String,
    pub magnet_link: String,
    pub file_name: String,
    pub category: usize,
    pub icon: CatIcon,
    pub trusted: bool,
    pub remake: bool,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum Sources {
    NyaaHtml,
    NyaaRss,
}

impl EnumIter<Sources> for Sources {
    fn iter() -> std::slice::Iter<'static, Sources> {
        static SORTS: &[Sources] = &[Sources::NyaaHtml, Sources::NyaaRss];
        SORTS.iter()
    }
}

impl ToString for Sources {
    fn to_string(&self) -> String {
        match self {
            Sources::NyaaHtml => "Nyaa HTML".to_owned(),
            Sources::NyaaRss => "Nyaa RSS".to_owned(),
        }
    }
}

pub trait Source {
    async fn search(app: &mut App, w: &Widgets) -> Result<Vec<Item>, Box<dyn Error>>;
    async fn sort(app: &mut App, w: &Widgets) -> Result<Vec<Item>, Box<dyn Error>>;
    async fn filter(app: &mut App, w: &Widgets) -> Result<Vec<Item>, Box<dyn Error>>;
    async fn categorize(app: &mut App, w: &Widgets) -> Result<Vec<Item>, Box<dyn Error>>;
}

pub async fn load(
    src: Sources,
    load_type: LoadType,
    app: &mut App,
    w: &Widgets,
) -> Result<Vec<Item>, Box<dyn Error>> {
    match src {
        Sources::NyaaHtml => match load_type {
            LoadType::Searching => NyaaHtmlSource::search(app, w).await,
            LoadType::Sorting => NyaaHtmlSource::sort(app, w).await,
            LoadType::Filtering => NyaaHtmlSource::filter(app, w).await,
            LoadType::Categorizing => NyaaHtmlSource::categorize(app, w).await,
            LoadType::Downloading => Ok(w.results.table.items.clone()),
        },
        Sources::NyaaRss => match load_type {
            LoadType::Searching => NyaaRssSource::search(app, w).await,
            LoadType::Sorting => NyaaRssSource::sort(app, w).await,
            LoadType::Filtering => NyaaRssSource::filter(app, w).await,
            LoadType::Categorizing => NyaaRssSource::categorize(app, w).await,
            LoadType::Downloading => Ok(w.results.table.items.clone()),
        },
    }
}
