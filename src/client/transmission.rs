use std::error::Error;

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
}

impl Default for TransmissionConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:9091/transmission/rpc".to_owned(),
            username: None,
            password: None,
        }
    }
}

async fn add_torrent(conf: &TransmissionConfig, link: String) -> Result<(), Box<dyn Error>> {
    let base_url = add_protocol(conf.base_url.clone(), false);
    let mut client =
        if let (Some(user), Some(password)) = (conf.username.clone(), conf.password.clone()) {
            let auth = BasicAuth { user, password };
            TransClient::with_auth(base_url.parse()?, auth)
        } else {
            TransClient::new(base_url.parse()?)
        };
    let add = TorrentAddArgs {
        filename: Some(link.clone()),
        ..TorrentAddArgs::default() // TODO: Add all options to config
    };
    match client.torrent_add(add).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string().into()),
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
    if let Err(e) = add_torrent(&conf, item.magnet_link.clone()).await {
        app.show_error(e);
    }
}
