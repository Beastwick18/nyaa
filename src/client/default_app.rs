use serde::{Deserialize, Serialize};

use crate::source::Item;

use super::{multidownload, ClientConfig, DownloadClient, DownloadError, DownloadResult};

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct DefaultAppConfig {
    use_magnet: bool,
}

pub struct DefaultAppClient;

pub fn load_config(cfg: &mut ClientConfig) {
    if cfg.default_app.is_none() {
        let def = DefaultAppConfig::default();
        cfg.default_app = Some(def);
    }
}

impl DownloadClient for DefaultAppClient {
    async fn download(item: Item, conf: ClientConfig, _: reqwest::Client) -> DownloadResult {
        let conf = match conf.default_app.to_owned() {
            Some(c) => c,
            None => {
                return DownloadResult::error(DownloadError(
                    "Failed to get default app config".to_owned(),
                ));
            }
        };
        let link = match conf.use_magnet {
            true => item.magnet_link.to_owned(),
            false => item.torrent_link.to_owned(),
        };
        let (success_ids, errors) =
            match open::that_detached(link).map_err(|e| DownloadError(e.to_string())) {
                Ok(()) => (vec![item.id], vec![]),
                Err(e) => (vec![], vec![DownloadError(e.to_string())]),
            };
        DownloadResult::new(
            "Successfully opened link in default app".to_owned(),
            success_ids,
            errors,
            false,
        )
    }

    async fn batch_download(
        items: Vec<Item>,
        conf: ClientConfig,
        client: reqwest::Client,
    ) -> DownloadResult {
        multidownload::<DefaultAppClient, _>(
            |s| format!("Successfully opened {} links in default app", s),
            &items,
            &conf,
            &client,
        )
        .await
    }
}
