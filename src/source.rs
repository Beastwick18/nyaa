use std::{collections::HashMap, error::Error, time::Duration};

use reqwest::Proxy;
use serde::{Deserialize, Serialize};

use crate::{
    app::{Context, LoadType, Widgets},
    config::Config,
    popup_enum,
    results::ResultTable,
    util::conv::add_protocol,
    widget::category::{CatEntry, CatIcon, CatStruct},
};

use self::{
    nyaa_html::{NyaaConfig, NyaaHtmlSource},
    nyaa_rss::NyaaRssSource,
    sukebei_nyaa::{SubekiHtmlSource, SukebeiNyaaConfig},
    torrent_galaxy::{TgxConfig, TorrentGalaxyHtmlSource},
};

pub mod nyaa_html;
pub mod nyaa_rss;
pub mod sukebei_nyaa;
pub mod torrent_galaxy;

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct SourceConfig {
    pub nyaa: Option<NyaaConfig>,
    #[serde(rename = "sukebei_nyaa")]
    pub sukebei: Option<SukebeiNyaaConfig>,
    #[serde(rename = "torrent_galaxy")]
    pub tgx: Option<TgxConfig>,
}

#[derive(Clone)]
pub struct SourceInfo {
    pub cats: Vec<CatStruct>,
    pub filters: Vec<String>,
    pub sorts: Vec<String>,
}

impl SourceInfo {
    pub fn entry_from_cfg(self, s: &str) -> CatEntry {
        for cat in self.cats.iter() {
            if let Some(ent) = cat.entries.iter().find(|ent| ent.cfg == s) {
                return ent.clone();
            }
        }
        self.cats[0].entries[0].clone()
    }

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
        ctx: &mut Context,
        w: &Widgets,
    ) -> Result<ResultTable, Box<dyn Error>>;
    async fn sort(
        client: &reqwest::Client,
        ctx: &mut Context,
        w: &Widgets,
    ) -> Result<ResultTable, Box<dyn Error>>;
    async fn filter(
        client: &reqwest::Client,
        ctx: &mut Context,
        w: &Widgets,
    ) -> Result<ResultTable, Box<dyn Error>>;
    async fn categorize(
        client: &reqwest::Client,
        ctx: &mut Context,
        w: &Widgets,
    ) -> Result<ResultTable, Box<dyn Error>>;
    fn info() -> SourceInfo;
    fn load_config(ctx: &mut Context);

    fn default_category(cfg: &Config) -> usize;
    fn default_sort(cfg: &Config) -> usize;
    fn default_filter(cfg: &Config) -> usize;
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

    pub fn load_config(self, ctx: &mut Context) {
        match self {
            Sources::NyaaHtml => NyaaHtmlSource::load_config(ctx),
            Sources::NyaaRss => NyaaRssSource::load_config(ctx),
            Sources::SubekiNyaa => SubekiHtmlSource::load_config(ctx),
            Sources::TorrentGalaxy => TorrentGalaxyHtmlSource::load_config(ctx),
        };
    }

    pub fn default_category(self, cfg: &Config) -> usize {
        match self {
            Sources::NyaaHtml => NyaaHtmlSource::default_category(cfg),
            Sources::NyaaRss => NyaaRssSource::default_category(cfg),
            Sources::SubekiNyaa => SubekiHtmlSource::default_category(cfg),
            Sources::TorrentGalaxy => TorrentGalaxyHtmlSource::default_category(cfg),
        }
    }

    pub fn default_sort(self, cfg: &Config) -> usize {
        match self {
            Sources::NyaaHtml => NyaaHtmlSource::default_sort(cfg),
            Sources::NyaaRss => NyaaRssSource::default_sort(cfg),
            Sources::SubekiNyaa => SubekiHtmlSource::default_sort(cfg),
            Sources::TorrentGalaxy => TorrentGalaxyHtmlSource::default_sort(cfg),
        }
    }

    pub fn default_filter(self, cfg: &Config) -> usize {
        match self {
            Sources::NyaaHtml => NyaaHtmlSource::default_filter(cfg),
            Sources::NyaaRss => NyaaRssSource::default_filter(cfg),
            Sources::SubekiNyaa => SubekiHtmlSource::default_filter(cfg),
            Sources::TorrentGalaxy => TorrentGalaxyHtmlSource::default_filter(cfg),
        }
    }
}
