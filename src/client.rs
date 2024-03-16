use std::error::Error;

use serde::{Deserialize, Serialize};

use crate::{app::App, source::Item, widget::EnumIter};

pub mod cmd;
pub mod qbit;
pub mod transmission;

#[derive(Clone, Copy, Deserialize, Serialize)]
pub enum Client {
    #[serde(rename = "cmd")]
    Cmd, // torrent_client_cmd
    #[serde(rename = "qBittorrent")]
    Qbit, // qBittorrent Web API
    #[serde(rename = "transmission")]
    Transmission, // qBittorrent Web API
}

impl EnumIter<Client> for Client {
    fn iter() -> std::slice::Iter<'static, Client> {
        static CLIENTS: &[Client] = &[Client::Cmd, Client::Qbit, Client::Transmission];
        CLIENTS.iter()
    }
}

impl ToString for Client {
    fn to_string(&self) -> String {
        match *self {
            Self::Cmd => "cmd".to_owned(),
            Self::Qbit => "qBittorrent".to_owned(),
            Self::Transmission => "transmission".to_owned(),
        }
    }
}

impl Client {
    pub async fn download(&self, item: &Item, app: &mut App) {
        match self {
            Self::Cmd => cmd::download(item, app).await,
            Self::Qbit => qbit::download(item, app).await,
            Self::Transmission => transmission::download(item, app).await,
        }
    }

    pub fn load_config(&self, app: &mut App) -> Result<(), Box<dyn Error>> {
        match self {
            Self::Cmd => cmd::load_config(app),
            Self::Qbit => qbit::load_config(app),
            Self::Transmission => transmission::load_config(app),
        };
        app.config.download_client = self.to_owned();
        app.config.clone().store()?;
        Ok(())
    }
}
