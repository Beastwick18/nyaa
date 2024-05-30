use serde::{Deserialize, Serialize};

use crate::{app::Context, source::Item, util::cmd::CommandBuilder};

use super::{multidownload, ClientConfig, DownloadClient, DownloadError, DownloadResult};

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct CmdConfig {
    cmd: String,
    shell_cmd: String,
}

pub struct CmdClient;

impl Default for CmdConfig {
    fn default() -> Self {
        CmdConfig {
            #[cfg(windows)]
            cmd: "curl \"{torrent}\" -o ~\\Downloads\\{file}".to_owned(),
            #[cfg(unix)]
            cmd: "curl \"{torrent}\" > ~/{file}".to_owned(),

            shell_cmd: CommandBuilder::default_shell(),
        }
    }
}

pub fn load_config(app: &mut Context) {
    if app.config.client.cmd.is_none() {
        app.config.client.cmd = Some(CmdConfig::default());
    }
}

impl DownloadClient for CmdClient {
    async fn download(item: Item, conf: ClientConfig, _: reqwest::Client) -> DownloadResult {
        let cmd = match conf.cmd.to_owned() {
            Some(c) => c,
            None => {
                return DownloadResult::error(DownloadError("Failed to get cmd config".to_owned()));
            }
        };
        let res = CommandBuilder::new(cmd.cmd)
            .sub("{magnet}", &item.magnet_link)
            .sub("{torrent}", &item.torrent_link)
            .sub("{title}", &item.title)
            .sub("{file}", &item.file_name)
            .run(cmd.shell_cmd)
            .map_err(|e| DownloadError(e.to_string()));

        let (success_ids, errors) = match res {
            Ok(()) => (vec![item.id], vec![]),
            Err(e) => (vec![], vec![DownloadError(e.to_string())]),
        };
        DownloadResult::new(
            "Successfully ran command".to_owned(),
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
        multidownload::<CmdClient, _>(
            |s| format!("Successfully ran command on {} torrents", s),
            &items,
            &conf,
            &client,
        )
        .await
    }
}
