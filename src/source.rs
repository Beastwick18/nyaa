use std::{error::Error, time::Duration};

use reqwest::Proxy;
use serde::{Deserialize, Serialize};

use crate::{
    app::{Context, LoadType, Widgets},
    popup_enum,
    util::conv::add_protocol,
    widget::{category::CatIcon, EnumIter},
};

use self::{nyaa_html::NyaaHtmlSource, nyaa_rss::NyaaRssSource};

pub mod nyaa_html;
pub mod nyaa_rss;

pub fn request_client(ctx: &Context) -> Result<reqwest::Client, reqwest::Error> {
    let mut client = reqwest::Client::builder()
        .gzip(true)
        .timeout(Duration::from_secs(ctx.config.timeout));
    if let Some(proxy_url) = ctx.config.request_proxy.to_owned() {
        client = client.proxy(Proxy::all(add_protocol(proxy_url, false))?);
    }
    client.build()
}

#[derive(Clone, Copy)]
pub enum ItemType {
    Trusted,
    Remake,
    None,
}

#[derive(Clone)]
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
}

popup_enum! {
    Sources;
    (0, NyaaHtml, "Nyaa HTML");
    (1, NyaaRss, "Nyaa RSS");
}

pub trait Source {
    async fn search(app: &mut Context, w: &Widgets) -> Result<Vec<Item>, Box<dyn Error>>;
    async fn sort(app: &mut Context, w: &Widgets) -> Result<Vec<Item>, Box<dyn Error>>;
    async fn filter(app: &mut Context, w: &Widgets) -> Result<Vec<Item>, Box<dyn Error>>;
    async fn categorize(app: &mut Context, w: &Widgets) -> Result<Vec<Item>, Box<dyn Error>>;
}

impl Sources {
    pub async fn load(
        &self,
        load_type: LoadType,
        app: &mut Context,
        w: &Widgets,
    ) -> Result<Vec<Item>, Box<dyn Error>> {
        match self {
            Sources::NyaaHtml => match load_type {
                LoadType::Searching => NyaaHtmlSource::search(app, w).await,
                LoadType::Sorting => NyaaHtmlSource::sort(app, w).await,
                LoadType::Filtering => NyaaHtmlSource::filter(app, w).await,
                LoadType::Categorizing => NyaaHtmlSource::categorize(app, w).await,
                LoadType::Downloading | LoadType::Batching => Ok(w.results.table.items.clone()),
            },
            Sources::NyaaRss => match load_type {
                LoadType::Searching => NyaaRssSource::search(app, w).await,
                LoadType::Sorting => NyaaRssSource::sort(app, w).await,
                LoadType::Filtering => NyaaRssSource::filter(app, w).await,
                LoadType::Categorizing => NyaaRssSource::categorize(app, w).await,
                LoadType::Downloading | LoadType::Batching => Ok(w.results.table.items.clone()),
            },
        }
    }
}
