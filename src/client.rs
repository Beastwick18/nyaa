use std::error::Error;

use serde::{Deserialize, Serialize};

use crate::{app::App, popup_enum, source::Item, widget::EnumIter};

use self::{cmd::CmdConfig, qbit::QbitConfig, transmission::TransmissionConfig};

pub mod cmd;
pub mod default_app;
pub mod qbit;
pub mod transmission;

popup_enum! {
    Client;

    #[serde(rename = "cmd")]
    (0, Cmd, "cmd");

    #[serde(rename = "qBittorrent")]
    (1, Qbit, "qBittorrent");

    #[serde(rename = "transmission")]
    (2, Transmission, "transmission");
}

#[derive(Default, Clone, Deserialize, Serialize)]
pub struct ClientConfig {
    #[serde(rename = "command")]
    pub cmd: Option<CmdConfig>,
    #[serde(rename = "qBittorrent")]
    pub qbit: Option<QbitConfig>,
    #[serde(rename = "transmission")]
    pub transmission: Option<TransmissionConfig>,
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
