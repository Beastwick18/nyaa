use std::{collections::HashMap, error::Error, fs};

use reqwest::{Response, StatusCode};
use serde::{Deserialize, Serialize};

use crate::{source::Item, util::conv::add_protocol};

use super::{ClientConfig, DownloadClient, DownloadError, DownloadResult};

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct QbitConfig {
    pub base_url: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub password_file: Option<String>,
    pub use_magnet: Option<bool>,
    pub savepath: Option<String>,
    pub category: Option<String>,  // Single category
    pub tags: Option<Vec<String>>, // Comma separated joined
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

pub struct QbitClient;

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
            username: None,
            password: None,
            password_file: None,
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

async fn login(
    qbit: &QbitConfig,
    client: &reqwest::Client,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let pass = match qbit.password.as_ref() {
        Some(pass) => Some(pass.to_owned()),
        None => match qbit.password_file.as_ref() {
            Some(file) => {
                let contents = fs::read_to_string(file)?;
                let expand = shellexpand::full(contents.trim())?;
                Some(expand.to_string())
            }
            None => None,
        },
    };
    if let (Some(user), Some(pass)) = (qbit.username.as_ref(), pass) {
        let base_url = add_protocol(qbit.base_url.clone(), false)?;
        let url = base_url.join("/api/v2/auth/login")?;
        let mut params = HashMap::new();
        params.insert("username", user);
        params.insert("password", &pass);
        let _ = client.post(url).form(&params).send().await?;
    }
    Ok(())
}

async fn logout(
    qbit: &QbitConfig,
    client: &reqwest::Client,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let base_url = add_protocol(qbit.base_url.clone(), false)?;
    let url = base_url.join("/api/v2/auth/logout")?;
    let _ = client.get(url).send().await;
    Ok(())
}

async fn add_torrent(
    qbit: &QbitConfig,
    links: String,
    client: &reqwest::Client,
) -> Result<Response, Box<dyn Error + Send + Sync>> {
    let base_url = add_protocol(qbit.base_url.clone(), false)?;
    let url = base_url.join("/api/v2/torrents/add")?;

    Ok(client.post(url).form(&qbit.to_form(links)).send().await?)
}

pub fn load_config(cfg: &mut ClientConfig) {
    if cfg.qbit.is_none() {
        cfg.qbit = Some(QbitConfig::default());
    }
}

impl DownloadClient for QbitClient {
    async fn download(item: Item, conf: ClientConfig, client: reqwest::Client) -> DownloadResult {
        let mut res = Self::batch_download(vec![item], conf, client).await;
        res.success_msg = Some("Successfully sent torrent to qBittorrent".to_string());
        res.batch = false;
        res
    }

    async fn batch_download(
        items: Vec<Item>,
        conf: ClientConfig,
        client: reqwest::Client,
    ) -> DownloadResult {
        // return DownloadResult::error(DownloadError("Failed to login :\\"));
        let Some(qbit) = conf.qbit.to_owned() else {
            return DownloadResult::error(DownloadError(
                "Failed to get qBittorrent config".to_owned(),
            ));
        };
        if let Some(labels) = qbit.tags.clone() {
            if let Some(bad) = labels.iter().find(|l| l.contains(',')) {
                let bad = format!("\"{}\"", bad);
                return DownloadResult::error(DownloadError(
                    format!("qBittorrent tags must not contain commas:\n{}", bad).to_owned(),
                ));
            }
        }
        if let Err(e) = login(&qbit, &client).await {
            return DownloadResult::error(DownloadError(format!("Failed to get SID:\n{}", e)));
        }
        let links = match qbit.use_magnet.unwrap_or(true) {
            true => items
                .iter()
                .map(|i| i.magnet_link.to_owned())
                .collect::<Vec<String>>()
                .join("\n"),
            false => items
                .iter()
                .map(|i| i.torrent_link.to_owned())
                .collect::<Vec<String>>()
                .join("\n"),
        };
        let res = match add_torrent(&qbit, links, &client).await {
            Ok(res) => res,
            Err(e) => {
                return DownloadResult::error(DownloadError(format!(
                    "Failed to get response:\n{}",
                    e
                )))
            }
        };
        if res.status() != StatusCode::OK {
            let mut msg = format!(
                "qBittorrent returned status code {} {}",
                res.status().as_u16(),
                res.status().canonical_reason().unwrap_or("")
            );
            if res.status() == StatusCode::FORBIDDEN {
                msg.push_str("\n\nLikely incorrect username/password");
            }
            return DownloadResult::error(DownloadError(msg));
        }

        let _ = logout(&qbit, &client).await;

        DownloadResult::new(
            format!("Successfully sent {} torrents to qBittorrent", items.len()),
            items.into_iter().map(|i| i.id).collect(),
            vec![],
            true,
        )
    }
}
