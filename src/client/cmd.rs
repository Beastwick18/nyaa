use serde::{Deserialize, Serialize};

use crate::{source::Item, util::cmd::CommandBuilder};

use super::{
    multidownload, BatchDownloadResult, ClientConfig, DownloadClient, SingleDownloadResult,
};

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

impl DownloadClient for CmdClient {
    async fn download(item: Item, conf: ClientConfig, _: reqwest::Client) -> SingleDownloadResult {
        let cmd = match conf.cmd.to_owned() {
            Some(c) => c,
            None => {
                return SingleDownloadResult::error("Failed to get cmd config");
            }
        };
        let res = CommandBuilder::new(cmd.cmd)
            .sub("{magnet}", &item.magnet_link)
            .sub("{torrent}", &item.torrent_link)
            .sub("{title}", &item.title)
            .sub("{file}", &item.file_name)
            .run(cmd.shell_cmd)
            .map_err(|e| e.to_string());

        match res {
            Ok(()) => SingleDownloadResult::success("Successfully ran command", item.id),
            Err(e) => SingleDownloadResult::error(e),
        }
    }

    async fn batch_download(
        items: Vec<Item>,
        conf: ClientConfig,
        client: reqwest::Client,
    ) -> BatchDownloadResult {
        multidownload::<CmdClient, _>(
            |s| format!("Successfully ran command on {} torrents", s),
            &items,
            &conf,
            &client,
        )
        .await
    }

    fn load_config(cfg: &mut ClientConfig) {
        if cfg.cmd.is_none() {
            cfg.cmd = Some(CmdConfig::default());
        }
    }
}
