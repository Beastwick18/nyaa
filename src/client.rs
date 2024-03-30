use std::error::Error;

use serde::{Deserialize, Serialize};

use crate::{app::App, popup_enum, source::Item, widget::EnumIter};

use self::{
    cmd::CmdConfig, default_app::DefaultAppConfig, download::DownloadConfig, qbit::QbitConfig,
    rqbit::RqbitConfig, transmission::TransmissionConfig,
};

pub mod cmd;
pub mod default_app;
pub mod download;
pub mod qbit;
pub mod rqbit;
pub mod transmission;

popup_enum! {
    Client;

    #[serde(rename = "qBittorrent")]
    (0, Qbit, "qBittorrent");

    #[serde(rename = "transmission")]
    (1, Transmission, "transmission");

    #[serde(rename = "rqbit")]
    (2, Rqbit, "rqbit");

    #[serde(rename = "default_app")]
    (3, DefaultApp, "Default App");

    #[serde(rename = "download")]
    (4, Download, "Download Torrent File");

    #[serde(rename = "command")]
    (5, Cmd, "Run Command");
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

impl Client {
    pub async fn download(&self, item: &Item, app: &mut App) {
        match self {
            Self::Cmd => cmd::download(item, app).await,
            Self::Qbit => qbit::download(item, app).await,
            Self::Transmission => transmission::download(item, app).await,
            Self::Rqbit => rqbit::download(item, app).await,
            Self::DefaultApp => default_app::download(item, app).await,
            Self::Download => download::download(item, app).await,
        }
    }

    // TODO: Add batch_download function
    // Downloads a Vec of &Item, all at once.

    pub fn load_config(&self, app: &mut App) -> Result<(), Box<dyn Error>> {
        match self {
            Self::Cmd => cmd::load_config(app),
            Self::Qbit => qbit::load_config(app),
            Self::Transmission => transmission::load_config(app),
            Self::Rqbit => rqbit::load_config(app),
            Self::DefaultApp => default_app::load_config(app),
            Self::Download => download::load_config(app),
        };
        app.config.download_client = self.to_owned();
        app.config.clone().store()?;
        Ok(())
    }
}
