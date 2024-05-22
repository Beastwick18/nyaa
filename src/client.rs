use serde::{Deserialize, Serialize};
use tokio::task::JoinSet;

use crate::{app::Context, popup_enum, source::Item};

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
    pub async fn download(&self, item: Item, ctx: &mut Context) {
        let conf = ctx.config.client.to_owned();
        let timeout = ctx.config.timeout;
        let item = item.clone();
        let result = match self {
            Self::Cmd => cmd::download(item, conf).await,
            Self::Qbit => qbit::download(item, conf, timeout).await,
            Self::Transmission => transmission::download(item, conf, timeout).await,
            Self::Rqbit => rqbit::download(item, conf, timeout).await,
            Self::DefaultApp => default_app::download(item, conf).await,
            Self::Download => download::download(item, conf, timeout).await,
        };
        match result {
            Ok(o) => ctx.notify(o),
            Err(e) => ctx.show_error(e),
        }
    }

    pub async fn download_async(
        self,
        item: Item,
        conf: ClientConfig,
        timeout: u64,
    ) -> Result<String, String> {
        let id = item.id.clone();
        let result = match self {
            Self::Cmd => cmd::download(item, conf).await,
            Self::Qbit => qbit::download(item, conf, timeout).await,
            Self::Transmission => transmission::download(item, conf, timeout).await,
            Self::Rqbit => rqbit::download(item, conf, timeout).await,
            Self::DefaultApp => default_app::download(item, conf).await,
            Self::Download => download::download(item, conf, timeout).await,
        };
        match result {
            Ok(_) => Ok(id),
            Err(e) => Err(e),
        }
    }

    async fn try_batch_download(
        self,
        items: Vec<Item>,
        conf: ClientConfig,
        timeout: u64,
    ) -> Option<Result<String, String>> {
        match self {
            Self::Qbit => Some(qbit::batch_download(items, conf, timeout).await),
            _ => None,
        }
    }

    pub async fn batch_download(&self, items: Vec<Item>, ctx: &mut Context) {
        let conf = ctx.config.client.to_owned();
        let timeout = ctx.config.timeout;

        if let Some(res) = self
            .try_batch_download(items.to_owned(), conf.to_owned(), timeout)
            .await
        {
            match res {
                Ok(o) => ctx.notify(o),
                Err(e) => ctx.show_error(e),
            }
        }

        let mut set = JoinSet::new();
        for item in items.iter() {
            let item = item.to_owned();
            set.spawn(self.download_async(item.to_owned(), conf.to_owned(), timeout));
        }
        let mut success_ids = vec![];
        while let Some(res) = set.join_next().await {
            let res = match res {
                Ok(res) => res,
                Err(e) => {
                    ctx.show_error(format!("Failed to join download thread:\n{}", e));
                    continue;
                }
            };
            match res {
                Ok(o) => {
                    success_ids.push(o);
                }
                Err(e) => {
                    ctx.show_error(e);
                }
            }
        }
        ctx.notify(format!(
            "Successfully downloaded {} torrents with {}",
            success_ids.len(),
            self.to_string()
        ));
        ctx.batch.retain(|i| !success_ids.contains(&i.id)); // Remove successes from batch
    }

    pub fn load_config(&self, app: &mut Context) {
        match self {
            Self::Cmd => cmd::load_config(app),
            Self::Qbit => qbit::load_config(app),
            Self::Transmission => transmission::load_config(app),
            Self::Rqbit => rqbit::load_config(app),
            Self::DefaultApp => default_app::load_config(app),
            Self::Download => download::load_config(app),
        };
        app.config.download_client = self.to_owned();
        // app.config.clone().store()?;
    }
}
