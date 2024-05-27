use std::{collections::HashMap, error::Error, time::Duration};

use reqwest::Proxy;
use serde::{Deserialize, Serialize};

use crate::{
    app::{Context, LoadType, Widgets},
    popup_enum,
    results::{ResultResponse, ResultTable},
    sync::SearchQuery,
    theme::Theme,
    util::conv::add_protocol,
    widget::category::{CatEntry, CatIcon, CatStruct},
};

use self::{
    nyaa_html::{NyaaConfig, NyaaHtmlSource},
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
    pub id: String,
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
    (0, Nyaa, "Nyaa");
    (1, SubekiNyaa, "Subeki");
    (2, TorrentGalaxy, "TorrentGalaxy");
}

pub trait Source {
    fn search(
        client: &reqwest::Client,
        search: &SearchQuery,
        config: &SourceConfig,
        date_format: Option<String>,
    ) -> impl std::future::Future<Output = Result<ResultResponse, Box<dyn Error + Send + Sync>>> + Send;
    fn sort(
        client: &reqwest::Client,
        search: &SearchQuery,
        config: &SourceConfig,
        date_format: Option<String>,
    ) -> impl std::future::Future<Output = Result<ResultResponse, Box<dyn Error + Send + Sync>>> + Send;
    fn filter(
        client: &reqwest::Client,
        search: &SearchQuery,
        config: &SourceConfig,
        date_format: Option<String>,
    ) -> impl std::future::Future<Output = Result<ResultResponse, Box<dyn Error + Send + Sync>>> + Send;
    fn categorize(
        client: &reqwest::Client,
        search: &SearchQuery,
        config: &SourceConfig,
        date_format: Option<String>,
    ) -> impl std::future::Future<Output = Result<ResultResponse, Box<dyn Error + Send + Sync>>> + Send;
    fn info() -> SourceInfo;
    fn load_config(config: &mut SourceConfig);

    fn default_category(config: &SourceConfig) -> usize;
    fn default_sort(config: &SourceConfig) -> usize;
    fn default_filter(config: &SourceConfig) -> usize;

    fn format_table(
        items: &[Item],
        sort: &SearchQuery,
        config: &SourceConfig,
        theme: &Theme,
    ) -> ResultTable;
}

impl Sources {
    pub async fn load(
        &self,
        load_type: LoadType,
        client: &reqwest::Client,
        search: &SearchQuery,
        config: &SourceConfig,
        date_format: Option<String>,
    ) -> Result<ResultResponse, Box<dyn Error + Send + Sync>> {
        match self {
            Sources::Nyaa => match load_type {
                LoadType::Searching | LoadType::Sourcing => {
                    NyaaHtmlSource::search(client, search, config, date_format).await
                }
                LoadType::Sorting => {
                    NyaaHtmlSource::sort(client, search, config, date_format).await
                }
                LoadType::Filtering => {
                    NyaaHtmlSource::filter(client, search, config, date_format).await
                }
                LoadType::Categorizing => {
                    NyaaHtmlSource::categorize(client, search, config, date_format).await
                }
                LoadType::Downloading | LoadType::Batching => unreachable!(),
            },
            Sources::SubekiNyaa => match load_type {
                LoadType::Searching | LoadType::Sourcing => {
                    SubekiHtmlSource::search(client, search, config, date_format).await
                }
                LoadType::Sorting => {
                    SubekiHtmlSource::sort(client, search, config, date_format).await
                }
                LoadType::Filtering => {
                    SubekiHtmlSource::filter(client, search, config, date_format).await
                }
                LoadType::Categorizing => {
                    SubekiHtmlSource::categorize(client, search, config, date_format).await
                }
                LoadType::Downloading | LoadType::Batching => unreachable!(),
            },
            Sources::TorrentGalaxy => match load_type {
                LoadType::Searching | LoadType::Sourcing => {
                    TorrentGalaxyHtmlSource::search(client, search, config, date_format).await
                }
                LoadType::Sorting => {
                    TorrentGalaxyHtmlSource::sort(client, search, config, date_format).await
                }
                LoadType::Filtering => {
                    TorrentGalaxyHtmlSource::filter(client, search, config, date_format).await
                }
                LoadType::Categorizing => {
                    TorrentGalaxyHtmlSource::categorize(client, search, config, date_format).await
                }
                LoadType::Downloading | LoadType::Batching => unreachable!(),
            },
        }
    }

    pub fn apply(self, ctx: &mut Context, w: &mut Widgets) {
        ctx.src_info = self.info();
        w.category.selected = self.default_category(&ctx.config.sources);
        w.category.major = 0;
        w.category.minor = 0;

        w.category.table.select(1);

        w.sort.selected.sort = self.default_sort(&ctx.config.sources);
        w.filter.selected = self.default_filter(&ctx.config.sources);

        // Go back to first page when changing source
        ctx.page = 1;
    }

    pub fn info(self) -> SourceInfo {
        match self {
            Sources::Nyaa => NyaaHtmlSource::info(),
            Sources::SubekiNyaa => SubekiHtmlSource::info(),
            Sources::TorrentGalaxy => TorrentGalaxyHtmlSource::info(),
        }
    }

    pub fn load_config(self, config: &mut SourceConfig) {
        match self {
            Sources::Nyaa => NyaaHtmlSource::load_config(config),
            Sources::SubekiNyaa => SubekiHtmlSource::load_config(config),
            Sources::TorrentGalaxy => TorrentGalaxyHtmlSource::load_config(config),
        };
    }

    pub fn default_category(self, config: &SourceConfig) -> usize {
        match self {
            Sources::Nyaa => NyaaHtmlSource::default_category(config),
            Sources::SubekiNyaa => SubekiHtmlSource::default_category(config),
            Sources::TorrentGalaxy => TorrentGalaxyHtmlSource::default_category(config),
        }
    }

    pub fn default_sort(self, config: &SourceConfig) -> usize {
        match self {
            Sources::Nyaa => NyaaHtmlSource::default_sort(config),
            Sources::SubekiNyaa => SubekiHtmlSource::default_sort(config),
            Sources::TorrentGalaxy => TorrentGalaxyHtmlSource::default_sort(config),
        }
    }

    pub fn default_filter(self, config: &SourceConfig) -> usize {
        match self {
            Sources::Nyaa => NyaaHtmlSource::default_filter(config),
            Sources::SubekiNyaa => SubekiHtmlSource::default_filter(config),
            Sources::TorrentGalaxy => TorrentGalaxyHtmlSource::default_filter(config),
        }
    }

    pub fn format_table(
        self,
        items: &[Item],
        search: &SearchQuery,
        config: &SourceConfig,
        theme: &Theme,
    ) -> ResultTable {
        match self {
            Sources::Nyaa => NyaaHtmlSource::format_table(items, search, config, theme),
            Sources::SubekiNyaa => SubekiHtmlSource::format_table(items, search, config, theme),
            Sources::TorrentGalaxy => {
                TorrentGalaxyHtmlSource::format_table(items, search, config, theme)
            }
        }
    }
}
