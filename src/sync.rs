use std::error::Error;

use crossterm::event::{self, Event};
use tokio::sync::mpsc;

use crate::{
    app::LoadType,
    client::{Client, ClientConfig, DownloadResult},
    results::Results,
    source::{Item, SourceConfig, SourceResponse, SourceResults, Sources},
    theme::Theme,
    widget::sort::SelectedSort,
};

pub trait EventSync {
    #[allow(clippy::too_many_arguments)]
    fn load_results(
        self,
        tx_res: mpsc::Sender<Result<SourceResults, Box<dyn Error + Send + Sync>>>,
        load_type: LoadType,
        src: Sources,
        client: reqwest::Client,
        search: SearchQuery,
        config: SourceConfig,
        theme: Theme,
        date_format: Option<String>,
    ) -> impl std::future::Future<Output = ()> + std::marker::Send + 'static;
    fn download(
        self,
        tx_dl: mpsc::Sender<DownloadResult>,
        batch: bool,
        items: Vec<Item>,
        config: ClientConfig,
        rq_client: reqwest::Client,
        client: Client,
    ) -> impl std::future::Future<Output = ()> + std::marker::Send + 'static;
    fn read_event_loop(
        self,
        tx_evt: mpsc::Sender<Event>,
    ) -> impl std::future::Future<Output = ()> + std::marker::Send + 'static;
}

#[derive(Clone)]
pub struct AppSync;

#[derive(Clone, Default)]
pub struct SearchQuery {
    pub query: String,
    pub page: usize,
    pub category: usize,
    pub filter: usize,
    pub sort: SelectedSort,
    pub user: Option<String>,
}

impl EventSync for AppSync {
    async fn load_results(
        self,
        tx_res: mpsc::Sender<Result<SourceResults, Box<dyn Error + Send + Sync>>>,
        load_type: LoadType,
        src: Sources,
        client: reqwest::Client,
        search: SearchQuery,
        config: SourceConfig,
        theme: Theme,
        date_format: Option<String>,
    ) {
        let res = src
            .load(load_type, &client, &search, &config, date_format)
            .await;
        let fmt = match res {
            Ok(SourceResponse::Results(res)) => Ok(SourceResults::Results(Results::new(
                search.clone(),
                res.clone(),
                src.format_table(&res.items, &search, &config, &theme),
            ))),
            Ok(SourceResponse::Captcha(c)) => Ok(SourceResults::Captcha(c)),
            Err(e) => Err(e),
        };
        let _ = tx_res.send(fmt).await;
    }

    async fn download(
        self,
        tx_dl: mpsc::Sender<DownloadResult>,
        batch: bool,
        items: Vec<Item>,
        config: ClientConfig,
        rq_client: reqwest::Client,
        client: Client,
    ) {
        let res = match batch {
            true => client.batch_download(items, config, rq_client).await,
            false => client.download(items[0].clone(), config, rq_client).await,
        };
        let _ = tx_dl.send(res).await;
    }

    async fn read_event_loop(self, tx_evt: mpsc::Sender<Event>) {
        loop {
            if let Ok(evt) = event::read() {
                let _ = tx_evt.send(evt).await;
            }
        }
    }
}
