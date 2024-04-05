use std::time::Duration;

use reqwest::Url;
use serde::{Deserialize, Serialize};
use transmission_rpc::{
    types::{BasicAuth, TorrentAddArgs},
    TransClient,
};

use crate::{app::Context, source::Item, util::add_protocol};

use super::ClientConfig;

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
    pub bandwidth_priority: Option<i64>,
}

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

async fn add_torrent(conf: &TransmissionConfig, link: String, timeout: u64) -> Result<(), String> {
    let base_url = add_protocol(conf.base_url.clone(), false);
    let url = match base_url.parse::<Url>() {
        Ok(url) => url,
        Err(e) => return Err(format!("Failed to parse base_url \"{}\":\n{}", base_url, e)),
    };
    let rq_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(timeout))
        .build();
    let rq_client = match rq_client {
        Ok(o) => o,
        Err(e) => {
            return Err(format!(
                "Failed to create request client for transmission:\n{}",
                e
            ))
        }
    };
    let mut client = TransClient::new_with_client(url.to_owned(), rq_client);
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

pub async fn download(item: Item, conf: ClientConfig, timeout: u64) -> Result<String, String> {
    let Some(conf) = conf.transmission.clone() else {
        return Err("Failed to get configuration for transmission".to_owned());
    };

    if let Some(labels) = conf.labels.clone() {
        if let Some(bad) = labels.iter().find(|l| l.contains(',')) {
            let bad = format!("\"{}\"", bad);
            return Err(format!(
                "Transmission labels must not contain commas:\n{}",
                bad
            ));
        }
    }

    let link = match conf.use_magnet {
        None | Some(true) => item.magnet_link.to_owned(),
        Some(false) => item.torrent_link.to_owned(),
    };
    match add_torrent(&conf, link, timeout).await {
        Ok(_) => Ok("Successfully sent torrent to Transmission".to_owned()),
        Err(e) => Err(e),
    }
}
