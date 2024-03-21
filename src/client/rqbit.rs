use std::{error::Error, time::Duration};

use reqwest::{Response, StatusCode, Url};
use serde::{Deserialize, Serialize};
use urlencoding::encode;

use crate::{
    app::App,
    source::{add_protocol, Item},
};

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct RqbitConfig {
    pub base_url: String,
    pub use_magnet: Option<bool>,
    pub overwrite: Option<bool>,
    pub output_folder: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RqbitForm {
    pub overwrite: Option<bool>,
    pub output_folder: Option<String>,
}

impl Default for RqbitConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:3030".to_owned(),
            use_magnet: None,
            overwrite: None,
            output_folder: None,
        }
    }
}

async fn add_torrent(
    conf: &RqbitConfig,
    link: String,
    timeout: Duration,
) -> Result<Response, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let base_url = add_protocol(conf.base_url.clone(), false);
    let mut url = Url::parse(&base_url)?.join("/torrents")?;
    let mut query: Vec<String> = vec![];
    if let Some(ow) = conf.overwrite {
        query.push(format!("overwrite={}", ow));
    }
    if let Some(out) = conf.output_folder.to_owned() {
        query.push(format!("output_folder={}", encode(&out)));
    }
    url.set_query(Some(&query.join("&")));

    match client.post(url).body(link).timeout(timeout).send().await {
        Ok(res) => Ok(res),
        Err(e) => Err(e.into()),
    }
}

pub fn load_config(app: &mut App) {
    if app.config.client.rqbit.is_none() {
        app.config.client.rqbit = Some(RqbitConfig::default());
    }
}

pub async fn download(item: &Item, app: &mut App) {
    load_config(app);
    let conf = match app.config.client.rqbit.clone() {
        Some(q) => q,
        None => {
            app.show_error("Failed to get rqbit config");
            return;
        }
    };
    let timeout = Duration::from_secs(app.config.timeout);
    let link = match conf.use_magnet.unwrap_or(true) {
        true => item.magnet_link.to_owned(),
        false => item.torrent_link.to_owned(),
    };
    let res = match add_torrent(&conf, link, timeout).await {
        Ok(r) => r,
        Err(e) => {
            app.show_error(format!("Failed to get response from rqbit\n{}", e));
            return;
        }
    };
    if res.status() != StatusCode::OK {
        app.show_error(format!(
            "rqbit returned status code {}",
            res.status().as_u16()
        ));
    }

    app.notify("Successfully sent torrent to rqbit");
}
