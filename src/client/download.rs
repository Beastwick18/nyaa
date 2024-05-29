use std::{error::Error, fs, path::PathBuf};

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{app::Context, source::Item};

use super::{multidownload, ClientConfig, DownloadClient, DownloadError, DownloadResult};

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct DownloadConfig {
    save_dir: String,
    filename: Option<String>,
}

pub struct DownloadFileClient;

impl Default for DownloadConfig {
    fn default() -> Self {
        let download_dir = match dirs::download_dir() {
            Some(p) => p,
            None => match dirs::home_dir() {
                Some(h) => h.join("Downloads"),
                None => PathBuf::from("./"),
            },
        };
        DownloadConfig {
            save_dir: download_dir.to_string_lossy().to_string(),
            filename: None,
        }
    }
}

pub fn load_config(app: &mut Context) {
    if app.config.client.download.is_none() {
        let def = DownloadConfig::default();
        app.config.client.download = Some(def);
    }
}

async fn download_torrent(
    torrent_link: String,
    filename: String,
    save_dir: String,
    client: reqwest::Client,
) -> Result<String, Box<dyn Error>> {
    let response = client.get(torrent_link.to_owned()).send().await?;
    if response.status() != StatusCode::OK {
        // Throw error if response code is not OK
        let code = response.status().as_u16();
        return Err(format!("{}\nInvalid response code: {}", torrent_link, code).into());
    }
    let content = response.bytes().await?;
    let mut buf = PathBuf::from(shellexpand::tilde(&save_dir).to_string());
    buf.push(filename);
    fs::write(buf.clone(), content)?;
    Ok(buf.to_string_lossy().to_string())
}

impl DownloadClient for DownloadFileClient {
    async fn download(item: Item, conf: ClientConfig, client: reqwest::Client) -> DownloadResult {
        let conf = match conf.download.to_owned() {
            Some(c) => c,
            None => {
                return DownloadResult::error(DownloadError(
                    "Failed to get download config".to_owned(),
                ));
            }
        };

        let filename = conf.filename.unwrap_or(item.file_name.to_owned());
        let (success_msg, success_ids, errors) = match download_torrent(
            item.torrent_link.to_owned(),
            filename,
            conf.save_dir.clone(),
            client,
        )
        .await
        {
            Ok(path) => (
                Some(format!("Saved to \"{}\"", path)),
                vec![item.id],
                vec![],
            ),
            Err(e) => (
                None,
                vec![],
                vec![DownloadError(
                    format!(
                        "Failed to download torrent to {}:\n{}",
                        conf.save_dir.to_owned(),
                        e
                    )
                    .to_owned(),
                )],
            ),
        };
        DownloadResult::new(success_msg, success_ids, errors, false)
    }

    async fn batch_download(
        items: Vec<Item>,
        conf: ClientConfig,
        client: reqwest::Client,
    ) -> DownloadResult {
        let save_dir = conf.download.clone().unwrap_or_default().save_dir.clone();
        multidownload::<DownloadFileClient, _>(
            |s| format!("Saved {} torrents to folder {}", s, save_dir),
            &items,
            &conf,
            &client,
        )
        .await
    }
}
