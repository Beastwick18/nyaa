use std::{error::Error, fs, path::PathBuf};

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{source::Item, util::conv::get_hash};

use super::{
    multidownload, BatchDownloadResult, ClientConfig, DownloadClient, SingleDownloadResult,
};

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct DownloadConfig {
    save_dir: String,
    filename: Option<String>,
    overwrite: bool,
    create_root_folder: bool,
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
            overwrite: true,
            create_root_folder: true,
        }
    }
}

async fn download_torrent(
    torrent_link: String,
    filename: String,
    save_dir: String,
    create_root_folder: bool,
    overwrite: bool,
    client: reqwest::Client,
) -> Result<String, Box<dyn Error>> {
    let response = client.get(torrent_link.to_owned()).send().await?;
    if response.status() != StatusCode::OK {
        // Throw error if response code is not OK
        let code = response.status().as_u16();
        return Err(format!("{}\nInvalid response code: {}", torrent_link, code).into());
    }
    let content = response.bytes().await?;
    let folder = PathBuf::from(shellexpand::full(&save_dir)?.to_string());
    let filepath = folder.join(filename);
    if !overwrite && filepath.exists() {
        return Err(format!(
            "{} already exists.\nEnable \"overwrite\" to overwrite files",
            filepath.to_string_lossy()
        )
        .into());
    }
    if create_root_folder && !folder.exists() {
        fs::create_dir_all(folder)?;
    }
    fs::write(filepath.clone(), content)?;
    Ok(filepath.to_string_lossy().to_string())
}

impl DownloadClient for DownloadFileClient {
    async fn download(
        item: Item,
        conf: ClientConfig,
        client: reqwest::Client,
    ) -> SingleDownloadResult {
        let conf = match conf.download.to_owned() {
            Some(c) => c,
            None => {
                return SingleDownloadResult::error("Failed to get download config");
            }
        };

        let filename = conf
            .filename
            .map(|f| {
                f.replace("{file}", &item.file_name)
                    .replace(
                        "{basename}",
                        item.file_name
                            .split_once(".torrent")
                            .map(|f| f.0)
                            .unwrap_or(&item.file_name),
                    )
                    .replace(
                        "{hash}",
                        &get_hash(item.magnet_link).unwrap_or("NO_HASH_FOUND".to_string()),
                    )
            })
            .unwrap_or(item.file_name.to_owned());
        match download_torrent(
            item.torrent_link.to_owned(),
            filename,
            conf.save_dir.clone(),
            conf.create_root_folder,
            conf.overwrite,
            client,
        )
        .await
        {
            Ok(path) => SingleDownloadResult::success(format!("Saved to \"{}\"", path), item.id),
            Err(e) => SingleDownloadResult::error(format!(
                "Failed to download torrent to {}:\n{}",
                conf.save_dir.to_owned(),
                e
            )),
        }
    }

    async fn batch_download(
        items: Vec<Item>,
        conf: ClientConfig,
        client: reqwest::Client,
    ) -> BatchDownloadResult {
        let save_dir = conf.download.clone().unwrap_or_default().save_dir.clone();
        multidownload::<DownloadFileClient, _>(
            |s| format!("Saved {} torrents to folder {}", s, save_dir),
            &items,
            &conf,
            &client,
        )
        .await
    }

    fn load_config(cfg: &mut ClientConfig) {
        if cfg.download.is_none() {
            cfg.download = Some(DownloadConfig::default());
        }
    }
}
