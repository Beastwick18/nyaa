use std::fmt::Display;

use serde::{Deserialize, Serialize};
use strum::{Display, VariantArray};
use tokio::task::JoinSet;

use crate::{client::cmd::CmdClient, source::Item, widget::notifications::Notification};

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

impl From<String> for DownloadError {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for DownloadError {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

pub trait DownloadClient {
    fn download(
        item: Item,
        conf: ClientConfig,
        client: reqwest::Client,
    ) -> impl std::future::Future<Output = SingleDownloadResult> + std::marker::Send + 'static;
    fn batch_download(
        items: Vec<Item>,
        conf: ClientConfig,
        client: reqwest::Client,
    ) -> impl std::future::Future<Output = BatchDownloadResult> + std::marker::Send + 'static;
    fn load_config(cfg: &mut ClientConfig);
}

impl Display for DownloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct DownloadSuccessResult {
    pub msg: Notification,
    pub id: String,
}

pub struct DownloadErrorResult {
    pub msg: Notification,
}

pub enum SingleDownloadResult {
    Success(DownloadSuccessResult),
    Error(DownloadErrorResult),
}

pub struct BatchDownloadResult {
    pub msg: Notification,
    pub errors: Vec<Notification>,
    pub ids: Vec<String>,
}

pub enum DownloadClientResult {
    Single(SingleDownloadResult),
    Batch(BatchDownloadResult),
}

impl SingleDownloadResult {
    pub fn success<S: Display>(msg: S, id: String) -> Self {
        Self::Success(DownloadSuccessResult {
            msg: Notification::success(msg),
            id,
        })
    }

    pub fn error<S: Display>(msg: S) -> Self {
        Self::Error(DownloadErrorResult {
            msg: Notification::error(msg),
        })
    }

    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success(_))
    }

    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error(_))
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
) -> BatchDownloadResult
where
    F: Fn(usize) -> String,
{
    let mut set = JoinSet::new();
    for item in items.iter() {
        let item = item.to_owned();
        set.spawn(C::download(item.clone(), conf.clone(), client.clone()));
    }

    let mut success_ids: Vec<String> = vec![];
    let mut errors: Vec<Notification> = vec![];
    while let Some(res) = set.join_next().await {
        match res.unwrap_or_else(|e| SingleDownloadResult::error(e)) {
            SingleDownloadResult::Success(sr) => success_ids.push(sr.id),
            SingleDownloadResult::Error(er) => errors.push(er.msg),
        }
    }

    BatchDownloadResult {
        msg: Notification::success(success_msg(success_ids.len())),
        errors,
        ids: success_ids,
    }
}

impl Client {
    pub async fn download(
        self,
        item: Item,
        conf: ClientConfig,
        client: reqwest::Client,
    ) -> SingleDownloadResult {
        match self {
            Self::Cmd => CmdClient::download(item, conf, client).await,
            Self::DefaultApp => DefaultAppClient::download(item, conf, client).await,
            Self::Download => DownloadFileClient::download(item, conf, client).await,
            Self::Qbit => QbitClient::download(item, conf, client).await,
            Self::Rqbit => RqbitClient::download(item, conf, client).await,
            Self::Transmission => TransmissionClient::download(item, conf, client).await,
        }
    }

    pub async fn batch_download(
        &self,
        items: Vec<Item>,
        conf: ClientConfig,
        client: reqwest::Client,
    ) -> BatchDownloadResult {
        match self {
            Self::Cmd => CmdClient::batch_download(items, conf, client).await,
            Self::DefaultApp => DefaultAppClient::batch_download(items, conf, client).await,
            Self::Download => DownloadFileClient::batch_download(items, conf, client).await,
            Self::Qbit => QbitClient::batch_download(items, conf, client).await,
            Self::Rqbit => RqbitClient::batch_download(items, conf, client).await,
            Self::Transmission => TransmissionClient::batch_download(items, conf, client).await,
        }
    }

    pub fn load_config(self, cfg: &mut ClientConfig) {
        match self {
            Self::Cmd => CmdClient::load_config(cfg),
            Self::DefaultApp => DefaultAppClient::load_config(cfg),
            Self::Download => DownloadFileClient::load_config(cfg),
            Self::Rqbit => RqbitClient::load_config(cfg),
            Self::Qbit => QbitClient::load_config(cfg),
            Self::Transmission => TransmissionClient::load_config(cfg),
        };
    }
}
