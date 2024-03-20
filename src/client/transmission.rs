use reqwest::Url;
use serde::{Deserialize, Serialize};
use transmission_rpc::{
    types::{BasicAuth, TorrentAddArgs},
    TransClient,
};

use crate::{
    app::App,
    source::{add_protocol, Item},
};

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

async fn add_torrent(conf: &TransmissionConfig, link: String) -> Result<(), String> {
    let base_url = add_protocol(conf.base_url.clone(), false);
    let url = match base_url.parse::<Url>() {
        Ok(url) => url,
        Err(e) => return Err(format!("Failed to parse base_url \"{}\":\n{}", base_url, e)),
    };
    let mut client = match (conf.username.clone(), conf.password.clone()) {
        (Some(user), Some(password)) => TransClient::with_auth(url, BasicAuth { user, password }),
        _ => TransClient::new(url),
    };
    let add = conf.clone().to_form(link);
    match client.torrent_add(add).await {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to add torrent:\n{}", e).into()),
    }
}

pub fn load_config(app: &mut App) {
    if app.config.client.transmission.is_none() {
        app.config.client.transmission = Some(TransmissionConfig::default());
    }
}

pub async fn download(item: &Item, app: &mut App) {
    let conf = match app.config.client.transmission.clone() {
        Some(c) => c,
        None => {
            app.show_error("Failed to get configuration for transmission");
            return;
        }
    };

    if let Some(labels) = conf.labels.clone() {
        if let Some(bad) = labels.iter().find(|l| l.contains(',')) {
            let bad = format!("\"{}\"", bad);
            app.show_error(format!(
                "Transmission labels must not contain commas:\n{}",
                bad
            ));
            return;
        }
    }

    let link = match conf.use_magnet {
        None | Some(true) => item.magnet_link.to_owned(),
        Some(false) => item.torrent_link.to_owned(),
    };
    if let Err(e) = add_torrent(&conf, link).await {
        app.show_error(e);
    }
}
