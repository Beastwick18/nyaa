use reqwest::Url;
use serde::{Deserialize, Serialize};
use transmission_rpc::{
    types::{BasicAuth, Priority, TorrentAddArgs},
    TransClient,
};

use crate::{app::Context, source::Item, util::conv::add_protocol};

use super::{multidownload, ClientConfig, DownloadClient, DownloadError, DownloadResult};

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct TransmissionConfig {
    pub base_url: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub use_magnet: Option<bool>,
    pub labels: Option<Vec<String>>,
    pub paused: Option<bool>,
    pub peer_limit: Option<i64>,
    pub download_dir: Option<String>,
    pub bandwidth_priority: Option<Priority>,
}

pub struct TransmissionClient;

impl Default for TransmissionConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:9091/transmission/rpc".to_owned(),
            username: None,
            password: None,
            use_magnet: None,
            labels: None,
            paused: None,
            peer_limit: None,
            download_dir: None,
            bandwidth_priority: None,
        }
    }
}

impl TransmissionConfig {
    fn to_form(&self, link: String) -> TorrentAddArgs {
        TorrentAddArgs {
            filename: Some(link),
            labels: self.labels.to_owned(),
            paused: self.paused,
            peer_limit: self.peer_limit,
            download_dir: self.download_dir.to_owned(),
            bandwidth_priority: self.bandwidth_priority,
            ..Default::default()
        }
    }
}

async fn add_torrent(
    conf: &TransmissionConfig,
    link: String,
    client: reqwest::Client,
) -> Result<(), String> {
    let base_url = add_protocol(conf.base_url.clone(), false);
    let url = match base_url.parse::<Url>() {
        Ok(url) => url,
        Err(e) => return Err(format!("Failed to parse base_url \"{}\":\n{}", base_url, e)),
    };
    let mut client = TransClient::new_with_client(url.to_owned(), client);
    if let (Some(user), Some(password)) = (conf.username.clone(), conf.password.clone()) {
        client.set_auth(BasicAuth { user, password });
    }
    let add = conf.clone().to_form(link);
    match client.torrent_add(add).await {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to add torrent:\n{}", e)),
    }
}

pub fn load_config(app: &mut Context) {
    if app.config.client.transmission.is_none() {
        app.config.client.transmission = Some(TransmissionConfig::default());
    }
}

impl DownloadClient for TransmissionClient {
    async fn download(item: Item, conf: ClientConfig, client: reqwest::Client) -> DownloadResult {
        let Some(conf) = conf.transmission.clone() else {
            return DownloadResult::error(DownloadError(
                "Failed to get configuration for transmission".to_owned(),
            ));
        };

        if let Some(labels) = conf.labels.clone() {
            if let Some(bad) = labels.iter().find(|l| l.contains(',')) {
                let bad = format!("\"{}\"", bad);
                return DownloadResult::error(DownloadError(format!(
                    "Transmission labels must not contain commas:\n{}",
                    bad
                )));
            }
        }

        let link = match conf.use_magnet {
            None | Some(true) => item.magnet_link.to_owned(),
            Some(false) => item.torrent_link.to_owned(),
        };
        if let Err(e) = add_torrent(&conf, link, client).await {
            return DownloadResult::error(DownloadError(e.to_string()));
        }
        DownloadResult::new(
            "Successfully sent torrent to Transmission".to_owned(),
            vec![item.id],
            vec![],
            false,
        )
    }

    async fn batch_download(
        items: Vec<Item>,
        conf: ClientConfig,
        client: reqwest::Client,
    ) -> DownloadResult {
        multidownload::<TransmissionClient, _>(
            |s| format!("Successfully sent {} torrents to rqbit", s),
            &items,
            &conf,
            &client,
        )
        .await
    }
}
