use std::{error::Error, fs};

use serde::{Deserialize, Serialize};
use transmission_rpc::{
    types::{BasicAuth, Priority, TorrentAddArgs},
    TransClient,
};

use crate::{source::Item, util::conv::add_protocol};

use super::{
    multidownload, BatchDownloadResult, ClientConfig, DownloadClient, SingleDownloadResult,
};

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct TransmissionConfig {
    pub base_url: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub password_file: Option<String>,
    pub use_magnet: Option<bool>,
    pub labels: Option<Vec<String>>,
    pub paused: Option<bool>,
    pub peer_limit: Option<i64>,
    pub download_dir: Option<String>,
    pub bandwidth_priority: Option<Priority>,
    pub yank_full_magnet: Option<bool>,
}

pub struct TransmissionClient;

impl Default for TransmissionConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:9091/transmission/rpc".to_owned(),
            username: None,
            password: None,
            password_file: None,
            use_magnet: None,
            labels: None,
            paused: None,
            peer_limit: None,
            download_dir: None,
            bandwidth_priority: None,
            yank_full_magnet: None,
        }
    }
}

impl TransmissionConfig {
    fn form(self, link: String) -> TorrentAddArgs {
        TorrentAddArgs {
            filename: Some(link),
            labels: self.labels,
            paused: self.paused,
            peer_limit: self.peer_limit,
            download_dir: self.download_dir,
            bandwidth_priority: self.bandwidth_priority,
            ..Default::default()
        }
    }
}

async fn add_torrent(
    conf: TransmissionConfig,
    link: String,
    client: reqwest::Client,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let base_url = add_protocol(conf.base_url.clone(), false)?;
    let mut client = TransClient::new_with_client(base_url, client);

    let pass = match conf.password.as_ref() {
        Some(pass) => Some(pass.clone()),
        None => match conf.password_file.as_ref() {
            Some(file) => {
                let contents = fs::read_to_string(file)?;
                let expand = shellexpand::full(contents.trim())?;
                Some(expand.to_string())
            }
            None => None,
        },
    };
    if let (Some(user), Some(password)) = (conf.username.as_ref(), pass.as_ref()) {
        client.set_auth(BasicAuth {
            user: user.clone(),
            password: password.clone(),
        });
    }
    let add = conf.form(link);
    client
        .torrent_add(add)
        .await
        .map_err(|e| format!("Failed to add torrent:\n{}", e))?;
    Ok(())
}

impl DownloadClient for TransmissionClient {
    async fn download(
        item: Item,
        conf: ClientConfig,
        client: reqwest::Client,
    ) -> SingleDownloadResult {
        let Some(conf) = conf.transmission.clone() else {
            return SingleDownloadResult::error("Failed to get configuration for transmission");
        };

        if let Some(labels) = conf.labels.clone() {
            if let Some(bad) = labels.iter().find(|l| l.contains(',')) {
                let bad = format!("\"{}\"", bad);
                return SingleDownloadResult::error(format!(
                    "Transmission labels must not contain commas:\n{}",
                    bad
                ));
            }
        }

        let link = super::Client::get_link(
            conf.use_magnet.unwrap_or(true),
            conf.yank_full_magnet,
            item.torrent_link.clone(),
            item.magnet_link.clone(),
        );
        if let Err(e) = add_torrent(conf, link, client).await {
            return SingleDownloadResult::error(e);
        }
        SingleDownloadResult::success("Successfully sent torrent to Transmission", item.id)
    }

    async fn batch_download(
        items: Vec<Item>,
        conf: ClientConfig,
        client: reqwest::Client,
    ) -> BatchDownloadResult {
        multidownload::<TransmissionClient, _>(
            |s| format!("Successfully sent {} torrents to Transmission", s),
            &items,
            &conf,
            &client,
        )
        .await
    }

    fn load_config(cfg: &mut ClientConfig) {
        if cfg.transmission.is_none() {
            cfg.transmission = Some(TransmissionConfig::default());
        }
    }
}
