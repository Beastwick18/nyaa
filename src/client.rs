use std::fmt::Display;

use serde::{Deserialize, Serialize};
use strum::{Display, VariantArray};
use tokio::task::JoinSet;

use crate::{client::cmd::CmdClient, source::Item};

use self::{
    cmd::CmdConfig,
    default_app::{DefaultAppClient, DefaultAppConfig},
    download::{DownloadConfig, DownloadFileClient},
    qbit::{QbitClient, QbitConfig},
    rqbit::{RqbitClient, RqbitConfig},
    transmission::{TransmissionClient, TransmissionConfig},
};

pub mod cmd;
pub mod default_app;
pub mod download;
pub mod qbit;
pub mod rqbit;
pub mod transmission;

pub struct DownloadError(String);

pub trait DownloadClient {
    fn download(
        item: Item,
        conf: ClientConfig,
        client: reqwest::Client,
    ) -> impl std::future::Future<Output = DownloadResult> + std::marker::Send + 'static;
    fn batch_download(
        items: Vec<Item>,
        conf: ClientConfig,
        client: reqwest::Client,
    ) -> impl std::future::Future<Output = DownloadResult> + std::marker::Send + 'static;
}

impl Display for DownloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct DownloadResult {
    pub success_msg: Option<String>,
    pub success_ids: Vec<String>,
    pub batch: bool,
    pub errors: Vec<DownloadError>,
}

impl DownloadResult {
    pub fn new<S: Into<Option<String>>>(
        success_msg: S,
        success_ids: Vec<String>,
        errors: Vec<DownloadError>,
        batch: bool,
    ) -> Self {
        DownloadResult {
            success_msg: success_msg.into(),
            success_ids,
            batch,
            errors,
        }
    }

    pub fn error(error: DownloadError) -> Self {
        DownloadResult {
            success_msg: None,
            success_ids: vec![],
            batch: false,
            errors: vec![error],
        }
    }
}

#[derive(Serialize, Deserialize, Display, Clone, Copy, VariantArray, PartialEq, Eq)]
pub enum Client {
    #[serde(rename = "qBittorrent")]
    #[strum(serialize = "qBittorrent")]
    Qbit = 0,

    #[serde(rename = "Transmission")]
    #[strum(serialize = "Transmission")]
    Transmission = 1,

    #[serde(rename = "rqbit")]
    #[strum(serialize = "rqbit")]
    Rqbit = 2,

    #[serde(rename = "DefaultApp")]
    #[strum(serialize = "Default App")]
    DefaultApp = 3,

    #[serde(rename = "DownloadTorrentFile")]
    #[strum(serialize = "Download Torrent File")]
    Download = 4,

    #[serde(rename = "RunCommand")]
    #[strum(serialize = "Run Command")]
    Cmd = 5,
}

#[derive(Default, Clone, Deserialize, Serialize)]
pub struct ClientConfig {
    #[serde(rename = "command")]
    pub cmd: Option<CmdConfig>,
    #[serde(rename = "qBittorrent")]
    pub qbit: Option<QbitConfig>,
    #[serde(rename = "transmission")]
    pub transmission: Option<TransmissionConfig>,
    #[serde(rename = "default_app")]
    pub default_app: Option<DefaultAppConfig>,
    #[serde(rename = "download")]
    pub download: Option<DownloadConfig>,
    #[serde(rename = "rqbit")]
    pub rqbit: Option<RqbitConfig>,
}

pub async fn multidownload<C: DownloadClient, F>(
    success_msg: F,
    items: &[Item],
    conf: &ClientConfig,
    client: &reqwest::Client,
) -> DownloadResult
where
    F: Fn(usize) -> String,
{
    let mut set = JoinSet::new();
    for item in items.iter() {
        let item = item.to_owned();
        set.spawn(C::download(item.clone(), conf.clone(), client.clone()));
    }
    let mut results: Vec<DownloadResult> = vec![];
    while let Some(res) = set.join_next().await {
        let res = match res {
            Ok(res) => res,
            Err(e) => {
                results.push(DownloadResult::error(DownloadError(e.to_string())));
                continue;
            }
        };
        results.push(res);
    }

    let (success, failure): (Vec<DownloadResult>, Vec<DownloadResult>) =
        results.into_iter().partition(|d| d.errors.is_empty());
    let success_ids = success.into_iter().fold(vec![], |acc, s| {
        acc.into_iter().chain(s.success_ids).collect()
    });
    let errors = failure
        .into_iter()
        .fold(vec![], |acc, s| acc.into_iter().chain(s.errors).collect());

    DownloadResult::new(success_msg(success_ids.len()), success_ids, errors, true)
}

impl Client {
    // pub async fn download(&self, item: Item, ctx: &mut Context) {
    //     let conf = ctx.config.client.to_owned();
    //     let timeout = ctx.config.timeout;
    //     let item = item.clone();
    //     let result = match self {
    //         Self::Cmd => cmd::download(item, conf).await,
    //         Self::Qbit => qbit::download(item, conf, timeout).await,
    //         Self::Transmission => transmission::download(item, conf, timeout).await,
    //         Self::Rqbit => rqbit::download(item, conf, timeout).await,
    //         Self::DefaultApp => default_app::download(item, conf).await,
    //         Self::Download => download::download(item, conf, timeout).await,
    //     };
    //     match result {
    //         Ok(o) => ctx.notify(o),
    //         Err(e) => ctx.show_error(e),
    //     }
    // }

    pub async fn download(
        self,
        item: Item,
        conf: ClientConfig,
        client: reqwest::Client,
    ) -> DownloadResult {
        match self {
            Self::Cmd => CmdClient::download(item, conf, client).await,
            Self::Qbit => QbitClient::download(item, conf, client).await,
            Self::Transmission => TransmissionClient::download(item, conf, client).await,
            Self::Rqbit => RqbitClient::download(item, conf, client).await,
            Self::DefaultApp => DefaultAppClient::download(item, conf, client).await,
            Self::Download => DownloadFileClient::download(item, conf, client).await,
        }
    }

    pub async fn batch_download(
        &self,
        items: Vec<Item>,
        conf: ClientConfig,
        client: reqwest::Client,
    ) -> DownloadResult {
        match self {
            Client::Cmd => CmdClient::batch_download(items, conf, client).await,
            Client::DefaultApp => DefaultAppClient::batch_download(items, conf, client).await,
            Client::Download => DownloadFileClient::batch_download(items, conf, client).await,
            Client::Rqbit => RqbitClient::batch_download(items, conf, client).await,
            Client::Qbit => QbitClient::batch_download(items, conf, client).await,
            Client::Transmission => TransmissionClient::batch_download(items, conf, client).await,
        }
        // let conf = ctx.config.client.to_owned();
        // let timeout = ctx.config.timeout;

        // if let Some(res) = self
        //     .try_batch_download(items.to_owned(), conf.to_owned(), timeout)
        //     .await
        // {
        //     return match res {
        //         Ok(o) => items
        //             .iter()
        //             .map(|i| DownloadResult::Success(i.title.to_owned(), o))
        //             .collect(),
        //         Err(e) => items
        //             .iter()
        //             .map(|i| DownloadResult::Failure(i.title.to_owned(), DownloadError(e.clone())))
        //             .collect(),
        //     };
        // }
        //
        // let mut set = JoinSet::new();
        // for item in items.iter() {
        //     let item = item.to_owned();
        //     set.spawn(self.download_async(item.to_owned(), conf.to_owned(), timeout));
        // }
        // let mut success_ids = vec![];
        // while let Some(res) = set.join_next().await {
        //     let res = match res {
        //         Ok(res) => res,
        //         Err(e) => {
        //             // ctx.show_error(format!("Failed to join download thread:\n{}", e));
        //             continue;
        //         }
        //     };
        //     match res {
        //         Ok(o) => {
        //             success_ids.push(o);
        //         }
        //         Err(e) => {
        //             // ctx.show_error(e);
        //         }
        //     }
        // }
        // vec![]
        // ctx.notify(format!(
        //     "Successfully downloaded {} torrents with {}",
        //     success_ids.len(),
        //     self,
        // ));
        // ctx.batch.retain(|i| !success_ids.contains(&i.id)); // Remove successes from batch
    }

    pub fn load_config(self, cfg: &mut ClientConfig) {
        match self {
            Self::Cmd => cmd::load_config(cfg),
            Self::Qbit => qbit::load_config(cfg),
            Self::Transmission => transmission::load_config(cfg),
            Self::Rqbit => rqbit::load_config(cfg),
            Self::DefaultApp => default_app::load_config(cfg),
            Self::Download => download::load_config(cfg),
        };
    }
}
