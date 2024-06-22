use std::error::Error;

use reqwest::{Response, StatusCode};
use serde::{Deserialize, Serialize};
use urlencoding::encode;

use crate::{source::Item, util::conv::add_protocol};

use super::{multidownload, ClientConfig, DownloadClient, DownloadError, DownloadResult};

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct RqbitConfig {
    pub base_url: String,
    pub use_magnet: Option<bool>,
    pub overwrite: Option<bool>,
    pub output_folder: Option<String>,
}

pub struct RqbitClient;

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
    client: &reqwest::Client,
) -> Result<Response, Box<dyn Error + Send + Sync>> {
    let base_url = add_protocol(conf.base_url.clone(), false)?;
    let mut url = base_url.join("/torrents")?;
    let mut query: Vec<String> = vec![];
    if let Some(ow) = conf.overwrite {
        query.push(format!("overwrite={}", ow));
    }
    if let Some(out) = conf.output_folder.to_owned() {
        query.push(format!("output_folder={}", encode(&out)));
    }
    url.set_query(Some(&query.join("&")));

    match client.post(url).body(link).send().await {
        Ok(res) => Ok(res),
        Err(e) => Err(e.into()),
    }
}

pub fn load_config(cfg: &mut ClientConfig) {
    if cfg.rqbit.is_none() {
        cfg.rqbit = Some(RqbitConfig::default());
    }
}

impl DownloadClient for RqbitClient {
    async fn download(item: Item, conf: ClientConfig, client: reqwest::Client) -> DownloadResult {
        let conf = match conf.rqbit.clone() {
            Some(q) => q,
            None => {
                return DownloadResult::error(DownloadError("Failed to get rqbit config".into()));
            }
        };
        let link = match conf.use_magnet.unwrap_or(true) {
            true => item.magnet_link.to_owned(),
            false => item.torrent_link.to_owned(),
        };
        let res = match add_torrent(&conf, link, &client).await {
            Ok(r) => r,
            Err(e) => {
                return DownloadResult::error(DownloadError(format!(
                    "Failed to get response from rqbit\n{}",
                    e
                )));
            }
        };
        if res.status() != StatusCode::OK {
            return DownloadResult::error(DownloadError(format!(
                "rqbit returned status code {}",
                res.status().as_u16()
            )));
        }

        DownloadResult::new(
            "Successfully sent torrent to rqbit".to_owned(),
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
        multidownload::<RqbitClient, _>(
            |s| format!("Successfully sent {} torrents to rqbit", s),
            &items,
            &conf,
            &client,
        )
        .await
    }
}
