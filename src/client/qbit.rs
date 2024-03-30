use std::{collections::HashMap, time::Duration};

use reqwest::{
    header::{COOKIE, REFERER, SET_COOKIE},
    Response, StatusCode,
};
use serde::{Deserialize, Serialize};

use crate::{
    app::Context,
    source::{add_protocol, Item},
};

use super::ClientConfig;

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct QbitConfig {
    pub base_url: String,
    pub username: String,
    pub password: String,
    pub use_magnet: Option<bool>,
    pub savepath: Option<String>,
    pub category: Option<String>,  // Single category
    pub tags: Option<Vec<String>>, // Comma seperated joined
    pub skip_checking: Option<bool>,
    pub paused: Option<bool>,
    pub create_root_folder: Option<bool>, // root_folder: String
    pub up_limit: Option<i32>,
    pub dl_limit: Option<i32>,
    pub ratio_limit: Option<f32>,
    pub seeding_time_limit: Option<i32>,
    pub auto_tmm: Option<bool>,
    pub sequential_download: Option<bool>,          // String
    pub prioritize_first_last_pieces: Option<bool>, // String
}

impl QbitConfig {
    fn to_form(&self, url: String) -> QbitForm {
        QbitForm {
            urls: url,
            savepath: self.savepath.to_owned(),
            category: self.category.to_owned(),
            tags: self.tags.clone().map(|v| v.join(",")),
            skip_checking: self.skip_checking.map(|b| b.to_string()),
            paused: self.paused.map(|b| b.to_string()),
            root_folder: self.create_root_folder.map(|b| b.to_string()),
            up_limit: self.up_limit,
            dl_limit: self.dl_limit,
            ratio_limit: self.ratio_limit,
            seeding_time_limit: self.seeding_time_limit,
            auto_tmm: self.auto_tmm,
            sequential_download: self.sequential_download.map(|b| b.to_string()),
            first_last_piece_prio: self.prioritize_first_last_pieces.map(|b| b.to_string()),
        }
    }
}

impl Default for QbitConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:8080".to_owned(),
            username: "admin".to_owned(),
            password: "adminadmin".to_owned(),
            use_magnet: None,
            savepath: None,
            category: None,
            tags: None,
            skip_checking: None,
            paused: None,
            create_root_folder: None,
            up_limit: None,
            dl_limit: None,
            ratio_limit: None,
            seeding_time_limit: None,
            auto_tmm: None,
            sequential_download: None,
            prioritize_first_last_pieces: None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct QbitForm {
    #[serde(rename = "urls")]
    urls: String,
    #[serde(rename = "savepath")]
    savepath: Option<String>,
    #[serde(rename = "category")]
    category: Option<String>,
    #[serde(rename = "tags")]
    tags: Option<String>,
    #[serde(rename = "skip_checking")]
    skip_checking: Option<String>,
    #[serde(rename = "paused")]
    paused: Option<String>,
    #[serde(rename = "root_folder")]
    root_folder: Option<String>,
    #[serde(rename = "upLimit")]
    up_limit: Option<i32>,
    #[serde(rename = "dlLimit")]
    dl_limit: Option<i32>,
    #[serde(rename = "ratioLimit")]
    ratio_limit: Option<f32>,
    #[serde(rename = "seedingTimeLimit")]
    seeding_time_limit: Option<i32>,
    #[serde(rename = "autoTMM")]
    auto_tmm: Option<bool>,
    #[serde(rename = "sequentialDownload")]
    sequential_download: Option<String>,
    #[serde(rename = "firstLastPiecePrio")]
    first_last_piece_prio: Option<String>,
    // torrents: Raw  // Disabled
    // cookie: String // Disabled
    // rename: String // Disabled
}

async fn login(qbit: &QbitConfig, timeout: Duration) -> Result<String, String> {
    let client = reqwest::Client::new();
    let base_url = add_protocol(qbit.base_url.clone(), false);
    let url = format!("{}/api/v2/auth/login", base_url);
    let mut params = HashMap::new();
    params.insert("username", qbit.username.to_owned());
    params.insert("password", qbit.password.to_owned());
    let res = client.post(url).form(&params).timeout(timeout).send().await;
    let res = res.map_err(|e| format!("Failed to send data to qBittorrent\n{}", e))?;
    let headers = res.headers().clone();
    let cookie = headers.get(SET_COOKIE).ok_or(format!(
        "Failed to get cookie from qBittorrent:\n{}",
        res.text().await.unwrap_or_default()
    ))?;

    let cookie = cookie
        .to_str()
        .map_err(|e| format!("Failed to parse cookie:\n{}", e))?;
    cookie
        .split(';')
        .find(|c| c.split_once('=').is_some_and(|s| s.0 == "SID"))
        .ok_or("No cookie returned by qBittorrent".to_owned())
        .map(|s| s.to_owned())
}

async fn logout(qbit: &QbitConfig, sid: String, timeout: Duration) {
    let client = reqwest::Client::new();
    let base_url = add_protocol(qbit.base_url.clone(), false);
    let _ = client
        .get(format!("{}/api/v2/auth/logout", base_url))
        .header(REFERER, base_url)
        .header(COOKIE, sid)
        .timeout(timeout)
        .send()
        .await;
}

async fn add_torrent(
    qbit: &QbitConfig,
    sid: String,
    link: String,
    timeout: Duration,
) -> Result<Response, reqwest::Error> {
    let client = reqwest::Client::new();
    let base_url = add_protocol(qbit.base_url.clone(), false);
    let url = format!("{}/api/v2/torrents/add", base_url);

    client
        .post(url)
        .header(REFERER, base_url)
        .header(COOKIE, sid)
        .form(&qbit.to_form(link))
        .timeout(timeout)
        .send()
        .await
}

pub fn load_config(app: &mut Context) {
    if app.config.client.qbit.is_none() {
        app.config.client.qbit = Some(QbitConfig::default());
    }
}

pub async fn download(item: Item, conf: ClientConfig, timeout: u64) -> Result<String, String> {
    let qbit = match conf.qbit.clone() {
        Some(q) => q,
        None => {
            return Err("Failed to get qBittorrent config".to_owned());
        }
    };
    if let Some(labels) = qbit.tags.clone() {
        if let Some(bad) = labels.iter().find(|l| l.contains(',')) {
            let bad = format!("\"{}\"", bad);
            return Err(format!("qBittorrent tags must not contain commas:\n{}", bad).to_owned());
        }
    }
    let timeout = Duration::from_secs(timeout);
    let sid = match login(&qbit, timeout).await {
        Ok(s) => s,
        Err(e) => {
            return Err(format!("Failed to get SID:\n{}", e));
        }
    };
    let link = match qbit.use_magnet.unwrap_or(true) {
        true => item.magnet_link.to_owned(),
        false => item.torrent_link.to_owned(),
    };
    let Ok(res) = add_torrent(&qbit, sid.to_owned(), link, timeout).await else {
        return Err("Failed to get response".to_owned());
    };
    if res.status() != StatusCode::OK {
        return Err(format!(
            "qBittorrent returned status code {}",
            res.status().as_u16()
        ));
    }

    logout(&qbit, sid.clone(), timeout).await;

    Ok("Successfully sent torrent to qBittorrent".to_owned())
}
