use std::{error::Error, time::Duration};

use reqwest::{Response, StatusCode, Url};
use serde::{Deserialize, Serialize};
use urlencoding::encode;

use crate::{app::Context, source::Item, util::conv::add_protocol};

use super::ClientConfig;

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

pub fn load_config(app: &mut Context) {
    if app.config.client.rqbit.is_none() {
        app.config.client.rqbit = Some(RqbitConfig::default());
    }
}

pub async fn download(item: Item, conf: ClientConfig, timeout: u64) -> Result<String, String> {
    let conf = match conf.rqbit.clone() {
        Some(q) => q,
        None => {
            return Err("Failed to get rqbit config".into());
        }
    };
    let timeout = Duration::from_secs(timeout);
    let link = match conf.use_magnet.unwrap_or(true) {
        true => item.magnet_link.to_owned(),
        false => item.torrent_link.to_owned(),
    };
    let res = match add_torrent(&conf, link, timeout).await {
        Ok(r) => r,
        Err(e) => {
            return Err(format!("Failed to get response from rqbit\n{}", e));
        }
    };
    if res.status() != StatusCode::OK {
        return Err(format!(
            "rqbit returned status code {}",
            res.status().as_u16()
        ));
    }

    Ok("Successfully sent torrent to rqbit".to_owned())
}
