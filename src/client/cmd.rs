use serde::{Deserialize, Serialize};

use crate::{app::Context, source::Item, util::cmd::CommandBuilder};

use super::ClientConfig;

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct CmdConfig {
    cmd: String,
    shell_cmd: String,
}

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

pub async fn download(item: Item, conf: ClientConfig) -> Result<String, String> {
    let cmd = match conf.cmd.to_owned() {
        Some(c) => c,
        None => {
            return Err("Failed to get cmd config".to_owned());
        }
    };
    match CommandBuilder::new(cmd.cmd)
        .sub("{magnet}", &item.magnet_link)
        .sub("{torrent}", &item.torrent_link)
        .sub("{title}", &item.title)
        .sub("{file}", &item.file_name)
        .run(cmd.shell_cmd)
    {
        Ok(_) => Ok("Successfully ran command".to_owned()),
        Err(e) => Err(e.to_string()),
    }
}
