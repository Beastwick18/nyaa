use std::{collections::HashMap, error::Error, time::Duration};

use reqwest::{header::SET_COOKIE, Response, StatusCode};
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
    pub use_magnet: bool,
}

impl Default for QbitConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:8080".to_owned(),
            username: "admin".to_owned(),
            password: "adminadmin".to_owned(),
            use_magnet: true,
        }
    }
}

async fn login(qbit: &QbitConfig, timeout: Duration) -> Result<String, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let base_url = add_protocol(qbit.base_url.clone(), false);
    let url = format!("{}/api/v2/auth/login", base_url);
    let mut params = HashMap::new();
    params.insert("username", qbit.username.to_owned());
    params.insert("password", qbit.password.to_owned());
    let res = client
        .post(url)
        .header("Referer", base_url)
        .form(&params)
        .timeout(timeout)
        .send()
        .await;
    if let Err(e) = res {
        return Err(format!("Failed to send data to qBittorrent\n{}", e).into());
    }
    let res = res.unwrap();
    let cookie = res.headers().get(SET_COOKIE);
    if cookie.is_none() {
        let headers = res.headers().clone();
        return Err(format!(
            "Failed to get cookie from qBittorrent:\n{:?}\n{}",
            headers.keys(),
            res.text().await.unwrap_or_default()
        )
        .into());
    }
    let cookie = cookie.unwrap().to_str();
    if let Err(e) = cookie {
        return Err(format!("Failed to parse cookie\n{}", e).into());
    }
    let cookie = cookie.unwrap().to_string();
    for c in cookie.split(';') {
        let split = c.trim().split_once('=');
        if let Some((name, value)) = split {
            if name == "SID" {
                return Ok(format!("SID={}", value));
            }
        }
    }
    Err("No cookie returned by qBittorrent".into())
}

async fn logout(qbit: &QbitConfig, sid: String, timeout: Duration) {
    let client = reqwest::Client::new();
    let base_url = add_protocol(qbit.base_url.clone(), false);
    let _ = client
        .get(format!("{}/api/v2/auth/logout", base_url))
        .header("Referer", base_url)
        .header("Cookie", sid)
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
    let mut params = HashMap::new();
    params.insert("urls", link);

    client
        .post(url)
        .header("Referer", base_url)
        .header("Cookie", sid)
        .form(&params)
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
        true => item.magnet_link.to_owned(),
        false => item.torrent_link.to_owned(),
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
