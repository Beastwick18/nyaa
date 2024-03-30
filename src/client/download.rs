use std::{error::Error, fs, path::PathBuf, time::Duration};

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{app::Context, source::Item};

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct DownloadConfig {
    save_dir: String,
    filename: Option<String>,
}

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
    timeout: u64,
) -> Result<String, Box<dyn Error>> {
    let client = reqwest::Client::builder()
        .gzip(true)
        .timeout(Duration::from_secs(timeout))
        .build()?;
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

pub async fn download(item: &Item, app: &mut Context) {
    load_config(app);
    let conf = match app.config.client.download.to_owned() {
        Some(c) => c,
        None => {
            app.show_error("Failed to get download config");
            return;
        }
    };

    let filename = conf.filename.unwrap_or(item.file_name.to_owned());
    match download_torrent(
        item.torrent_link.to_owned(),
        filename,
        conf.save_dir.clone(),
        app.config.timeout,
    )
    .await
    {
        Ok(path) => app.notify(format!("Saved to \"{}\"", path)),
        Err(e) => app.show_error(format!(
            "Failed to download torrent to {}:\n{}",
            conf.save_dir.to_owned(),
            e
        )),
    }
}
