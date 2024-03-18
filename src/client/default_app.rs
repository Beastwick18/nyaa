use serde::{Deserialize, Serialize};

use crate::{app::App, source::Item};

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct DefaultAppConfig {
    use_magnet: Option<bool>,
}

pub fn load_config(app: &mut App) {
    if app.config.client.default_app.is_none() {
        let def = DefaultAppConfig::default();
        app.config.client.default_app = Some(def);
    }
}

pub async fn download(item: &Item, app: &mut App) {
    load_config(app);
    let conf = match app.config.client.default_app.to_owned() {
        Some(c) => c,
        None => {
            app.show_error("Failed to get default app config");
            return;
        }
    };
    let link = match conf.use_magnet {
        None | Some(true) => item.magnet_link.to_owned(),
        Some(false) => item.torrent_link.to_owned(),
    };
    if let Err(e) = open::that_detached(&link) {
        app.show_error(format!("Unable to open {}:\n{}", link, e));
    }
}
