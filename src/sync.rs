use std::{
    error::Error,
    fs,
    path::PathBuf,
    time::{Duration, SystemTime},
};

use crossterm::event::{self, Event};
use tokio::sync::mpsc;

use crate::{
    app::LoadType,
    client::{Client, ClientConfig, DownloadResult},
    config::CONFIG_FILE,
    results::Results,
    source::{Item, SourceConfig, SourceResponse, SourceResults, Sources},
    theme::{Theme, THEMES_PATH},
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
    fn watch_config_loop(
        self,
        tx_evt: mpsc::Sender<ReloadType>,
    ) -> impl std::future::Future<Output = ()> + std::marker::Send + 'static;
}

#[derive(Clone)]
pub struct AppSync {
    config_path: PathBuf,
}

impl AppSync {
    pub fn new(config_path: PathBuf) -> Self {
        Self { config_path }
    }
}

#[derive(Clone, Default)]
pub struct SearchQuery {
    pub query: String,
    pub page: usize,
    pub category: usize,
    pub filter: usize,
    pub sort: SelectedSort,
    pub user: Option<String>,
}

#[derive(Clone)]
pub enum ReloadType {
    Config,
    Theme(String),
}

fn watch(path: &PathBuf, last_modified: SystemTime) -> bool {
    if let Ok(meta) = fs::metadata(path) {
        if let Ok(time) = meta.modified() {
            if time > last_modified {
                return true;
            }
        }
    }
    false
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
            #[cfg(feature = "captcha")]
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

    async fn watch_config_loop(self, tx_cfg: mpsc::Sender<ReloadType>) {
        let config_path = self.config_path.clone();
        let config_file = config_path.join(CONFIG_FILE);
        let themes_path = config_path.join(THEMES_PATH);
        let now = SystemTime::now();

        let mut last_modified = now;
        loop {
            if watch(&config_file, last_modified) {
                last_modified = SystemTime::now();
                let _ = tx_cfg.send(ReloadType::Config).await;
            }
            let theme_files = fs::read_dir(&themes_path).ok().and_then(|v| {
                v.filter_map(Result::ok)
                    .map(|v| v.path())
                    .find(|p| watch(&p, last_modified))
            });
            if let Some(theme) = theme_files {
                last_modified = SystemTime::now();
                let _ = tx_cfg
                    .send(ReloadType::Theme(
                        theme
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string(),
                    ))
                    .await;
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }
}
