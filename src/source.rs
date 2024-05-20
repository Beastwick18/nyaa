use std::{collections::HashMap, error::Error, time::Duration};

use reqwest::Proxy;

use crate::{
    app::{Context, LoadType, Widgets},
    popup_enum,
    results::ResultTable,
    util::conv::add_protocol,
    widget::category::{CatEntry, CatIcon, CatStruct},
};

use self::{
    nyaa_html::NyaaHtmlSource, nyaa_rss::NyaaRssSource, sukebei_nyaa::SubekiHtmlSource,
    torrent_galaxy::TorrentGalaxyHtmlSource,
};

pub mod nyaa_html;
pub mod nyaa_rss;
pub mod sukebei_nyaa;
pub mod torrent_galaxy;

#[derive(Clone)]
pub struct SourceInfo {
    pub cats: Vec<CatStruct>,
    pub filters: Vec<String>,
    pub sorts: Vec<String>,
}

impl SourceInfo {
    pub fn entry_from_str(self, s: &str) -> CatEntry {
        let split: Vec<&str> = s.split('_').collect();
        let high = split.first().unwrap_or(&"1").parse().unwrap_or(1);
        let low = split.last().unwrap_or(&"0").parse().unwrap_or(0);
        let id = high * 10 + low;
        self.entry_from_id(id)
    }

    pub fn entry_from_id(self, id: usize) -> CatEntry {
        for cat in self.cats.iter() {
            if let Some(ent) = cat.entries.iter().find(|ent| ent.id == id) {
                return ent.clone();
            }
        }
        self.cats[0].entries[0].clone()
    }

    // pub fn find_category<S: Into<String>>(&self, name: S) -> Option<CatEntry> {
    //     let name = name.into();
    //     for cat in self.cats.iter() {
    //         if let Some(ent) = cat
    //             .entries
    //             .iter()
    //             .find(|ent| ent.cfg.eq_ignore_ascii_case(&name))
    //         {
    //             return Some(ent.to_owned());
    //         }
    //     }
    //     None
    // }
}

pub fn request_client(ctx: &Context) -> Result<reqwest::Client, reqwest::Error> {
    let mut client = reqwest::Client::builder()
        .gzip(true)
        .timeout(Duration::from_secs(ctx.config.timeout));
    if let Some(proxy_url) = ctx.config.request_proxy.to_owned() {
        client = client.proxy(Proxy::all(add_protocol(proxy_url, false))?);
    }
    client.build()
}

#[derive(Default, Clone, Copy)]
pub enum ItemType {
    #[default]
    None,
    Trusted,
    Remake,
}

#[derive(Clone, Default)]
pub struct Item {
    pub id: usize,
    pub date: String,
    pub seeders: u32,
    pub leechers: u32,
    pub downloads: u32,
    pub size: String,
    pub bytes: usize,
    pub title: String,
    pub torrent_link: String,
    pub magnet_link: String,
    pub post_link: String,
    pub file_name: String,
    pub category: usize,
    pub icon: CatIcon,
    pub item_type: ItemType,
    pub extra: HashMap<String, String>,
}

popup_enum! {
    Sources;
    (0, NyaaHtml, "Nyaa HTML");
    (1, NyaaRss, "Nyaa RSS");
    (2, SubekiNyaa, "Subeki");
    (3, TorrentGalaxy, "TorrentGalaxy");
}

pub trait Source {
    async fn search(
        client: &reqwest::Client,
        app: &mut Context,
        w: &Widgets,
    ) -> Result<ResultTable, Box<dyn Error>>;
    async fn sort(
        client: &reqwest::Client,
        app: &mut Context,
        w: &Widgets,
    ) -> Result<ResultTable, Box<dyn Error>>;
    async fn filter(
        client: &reqwest::Client,
        app: &mut Context,
        w: &Widgets,
    ) -> Result<ResultTable, Box<dyn Error>>;
    async fn categorize(
        client: &reqwest::Client,
        app: &mut Context,
        w: &Widgets,
    ) -> Result<ResultTable, Box<dyn Error>>;
    fn info() -> SourceInfo;
    // fn default_category() -> usize;
}

impl Sources {
    pub async fn load(
        &self,
        client: &reqwest::Client,
        load_type: LoadType,
        ctx: &mut Context,
        w: &Widgets,
    ) -> Option<Result<ResultTable, Box<dyn Error>>> {
        match self {
            Sources::NyaaHtml => match load_type {
                LoadType::Searching | LoadType::Sourcing => {
                    Some(NyaaHtmlSource::search(client, ctx, w).await)
                }
                LoadType::Sorting => Some(NyaaHtmlSource::sort(client, ctx, w).await),
                LoadType::Filtering => Some(NyaaHtmlSource::filter(client, ctx, w).await),
                LoadType::Categorizing => Some(NyaaHtmlSource::categorize(client, ctx, w).await),
                LoadType::Downloading | LoadType::Batching => None,
            },
            Sources::NyaaRss => match load_type {
                LoadType::Searching | LoadType::Sourcing => {
                    Some(NyaaRssSource::search(client, ctx, w).await)
                }
                LoadType::Sorting => Some(NyaaRssSource::sort(client, ctx, w).await),
                LoadType::Filtering => Some(NyaaRssSource::filter(client, ctx, w).await),
                LoadType::Categorizing => Some(NyaaRssSource::categorize(client, ctx, w).await),
                LoadType::Downloading | LoadType::Batching => None,
            },
            Sources::SubekiNyaa => match load_type {
                LoadType::Searching | LoadType::Sourcing => {
                    Some(SubekiHtmlSource::search(client, ctx, w).await)
                }
                LoadType::Sorting => Some(SubekiHtmlSource::sort(client, ctx, w).await),
                LoadType::Filtering => Some(SubekiHtmlSource::filter(client, ctx, w).await),
                LoadType::Categorizing => Some(SubekiHtmlSource::categorize(client, ctx, w).await),
                LoadType::Downloading | LoadType::Batching => None,
            },
            Sources::TorrentGalaxy => match load_type {
                LoadType::Searching | LoadType::Sourcing => {
                    Some(TorrentGalaxyHtmlSource::search(client, ctx, w).await)
                }
                LoadType::Sorting => Some(TorrentGalaxyHtmlSource::sort(client, ctx, w).await),
                LoadType::Filtering => Some(TorrentGalaxyHtmlSource::filter(client, ctx, w).await),
                LoadType::Categorizing => {
                    Some(TorrentGalaxyHtmlSource::categorize(client, ctx, w).await)
                }
                LoadType::Downloading | LoadType::Batching => None,
            },
        }
    }

    pub fn info(self) -> SourceInfo {
        match self {
            Sources::NyaaHtml => NyaaHtmlSource::info(),
            Sources::NyaaRss => NyaaRssSource::info(),
            Sources::SubekiNyaa => SubekiHtmlSource::info(),
            Sources::TorrentGalaxy => TorrentGalaxyHtmlSource::info(),
        }
    }

    // pub fn default_category(self) -> usize {
    //     match self {
    //         Sources::NyaaHtml => NyaaHtmlSource::default_category(),
    //         Sources::NyaaRss => NyaaRssSource::default_category(),
    //         Sources::SubekiNyaa => SubekiHtmlSource::default_category(),
    //         Sources::TorrentGalaxy => TorrentGalaxyHtmlSource::default_category(),
    //     }
    // }
    //
    // pub fn default_sort(self) -> usize {
    //     match self {
    //         Sources::NyaaHtml => NyaaHtmlSource::default_category(),
    //         Sources::NyaaRss => NyaaRssSource::default_category(),
    //         Sources::SubekiNyaa => SubekiHtmlSource::default_category(),
    //         Sources::TorrentGalaxy => TorrentGalaxyHtmlSource::default_category(),
    //     }
    // }
    //
    // pub fn default_filter(self) -> usize {
    //     match self {
    //         Sources::NyaaHtml => NyaaHtmlSource::default_category(),
    //         Sources::NyaaRss => NyaaRssSource::default_category(),
    //         Sources::SubekiNyaa => SubekiHtmlSource::default_category(),
    //         Sources::TorrentGalaxy => TorrentGalaxyHtmlSource::default_category(),
    //     }
    // }
}
