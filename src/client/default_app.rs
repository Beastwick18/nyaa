use serde::{Deserialize, Serialize};

use crate::source::Item;

use super::{
    multidownload, BatchDownloadResult, ClientConfig, DownloadClient, SingleDownloadResult,
};

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct DefaultAppConfig {
    use_magnet: bool,
}

pub struct DefaultAppClient;

impl DownloadClient for DefaultAppClient {
    async fn download(item: Item, conf: ClientConfig, _: reqwest::Client) -> SingleDownloadResult {
        let conf = match conf.default_app.to_owned() {
            Some(c) => c,
            None => {
                return SingleDownloadResult::error("Failed to get default app config");
            }
        };
        let link = match conf.use_magnet {
            true => item.magnet_link.to_owned(),
            false => item.torrent_link.to_owned(),
        };
        match open::that_detached(link).map_err(|e| e.to_string()) {
            Ok(()) => {
                SingleDownloadResult::success("Successfully opened link in default app", item.id)
            }
            Err(e) => SingleDownloadResult::error(e),
        }
    }

    async fn batch_download(
        items: Vec<Item>,
        conf: ClientConfig,
        client: reqwest::Client,
    ) -> BatchDownloadResult {
        multidownload::<DefaultAppClient, _>(
            |s| format!("Successfully opened {} links in default app", s),
            &items,
            &conf,
            &client,
        )
        .await
    }

    fn load_config(cfg: &mut ClientConfig) {
        if cfg.default_app.is_none() {
            let def = DefaultAppConfig::default();
            cfg.default_app = Some(def);
        }
    }
}
