use serde::{Deserialize, Serialize};

use crate::{app::Context, source::Item};

use super::ClientConfig;

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct DefaultAppConfig {
    use_magnet: Option<bool>,
}

pub fn load_config(app: &mut Context) {
    if app.config.client.default_app.is_none() {
        let def = DefaultAppConfig::default();
        app.config.client.default_app = Some(def);
    }
}

pub async fn download(item: Item, conf: ClientConfig) -> Result<String, String> {
    let conf = match conf.default_app.to_owned() {
        Some(c) => c,
        None => {
            return Err("Failed to get default app config".to_owned());
        }
    };
    let link = match conf.use_magnet {
        None | Some(true) => item.magnet_link.to_owned(),
        Some(false) => item.torrent_link.to_owned(),
    };
    match open::that_detached(&link) {
        Ok(_) => Ok("Successfully opened link in default app".to_owned()),
        Err(e) => Err(format!("Unable to open {}:\n{}", link, e).to_owned()),
    }
}
