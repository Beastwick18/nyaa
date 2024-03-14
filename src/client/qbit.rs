use std::{collections::HashMap, time::Duration};

use reqwest::{
    header::{COOKIE, REFERER, SET_COOKIE},
    Response, StatusCode,
};
use serde::{Deserialize, Serialize};

use crate::{
    app::App,
    source::{add_protocol, Item},
};

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct QbitConfig {
    pub base_url: String,
    pub username: String,
    pub password: String,
    pub use_magnet: Option<bool>,
    pub savepath: Option<String>,
    pub category: Option<String>, // Single category
    pub tags: Option<String>,     // Comma seperated
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

fn bool_str(b: Option<bool>) -> Option<String> {
    b.map(|x| match x {
        true => "true".to_owned(),
        false => "false".to_owned(),
    })
}

impl QbitConfig {
    fn to_form(&self, url: String) -> QbitForm {
        QbitForm {
            urls: url,
            savepath: self.savepath.to_owned(),
            category: self.category.to_owned(),
            tags: self.tags.to_owned(),
            skip_checking: bool_str(self.skip_checking),
            paused: bool_str(self.paused),
            root_folder: bool_str(self.create_root_folder),
            up_limit: self.up_limit,
            dl_limit: self.dl_limit,
            ratio_limit: self.ratio_limit,
            seeding_time_limit: self.seeding_time_limit,
            auto_tmm: self.auto_tmm,
            sequential_download: bool_str(self.sequential_download),
            first_last_piece_prio: bool_str(self.prioritize_first_last_pieces),
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
        .map_err(|e| format!("Failed to parse cookie\n{}", e))?;
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
    // let mut params = HashMap::new();
    // params.insert("urls", link);
    // params.insert("category", "Test category".to_owned());
    // params.insert("test", 0.2);

    client
        .post(url)
        .header(REFERER, base_url)
        .header(COOKIE, sid)
        .form(&qbit.to_form(link))
        .timeout(timeout)
        .send()
        .await
}

pub fn load_config(app: &mut App) {
    if app.config.qbit.is_none() {
        app.config.qbit = Some(QbitConfig::default());
    }
}

pub async fn download(item: &Item, app: &mut App) {
    let qbit = app.config.qbit.clone().unwrap();
    let timeout = Duration::from_secs(app.config.timeout);
    let sid = match login(&qbit, timeout).await {
        Ok(s) => s,
        Err(e) => {
            app.show_error(format!("Failed to get SID:\n{}", e));
            return;
        }
    };
    let link = match qbit.use_magnet {
        None | Some(true) => item.magnet_link.to_owned(),
        Some(false) => item.torrent_link.to_owned(),
    };
    let Ok(res) = add_torrent(&qbit, sid.to_owned(), link, timeout).await else {
        app.show_error("Failed to get response");
        return;
    };
    if res.status() != StatusCode::OK {
        app.show_error(format!(
            "qBittorrent returned status code {}",
            res.status().as_u16()
        ));
        return;
    }

    logout(&qbit, sid.clone(), timeout).await;
}
