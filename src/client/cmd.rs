use serde::{Deserialize, Serialize};

use crate::{app::Context, source::Item, util::CommandBuilder};

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
            // #[cfg(windows)]
            // shell_cmd: "powershell.exe -Command".to_owned(),
            // #[cfg(unix)]
            // shell_cmd: "sh -c".to_owned(),
        }
    }
}

pub fn load_config(app: &mut Context) {
    if app.config.client.cmd.is_none() {
        let mut def = CmdConfig::default();
        // Replace deprecated torrent_client_cmd with client.command config
        if let Some(cmd) = app.config.torrent_client_cmd.clone() {
            def.cmd = cmd;
        }
        app.config.client.cmd = Some(def);
        app.config.torrent_client_cmd = None;
    }
}

pub async fn download(item: Item, conf: ClientConfig) -> Result<String, String> {
    // load_config(app);
    let cmd = match conf.cmd.to_owned() {
        Some(c) => c,
        None => {
            return Err("Failed to get cmd config".to_owned());
        }
    };
    CommandBuilder::new(cmd.cmd)
        .sub("{magnet}", &item.magnet_link)
        .sub("{torrent}", &item.torrent_link)
        .sub("{title}", &item.title)
        .sub("{file}", &item.file_name)
        .run(cmd.shell_cmd)
    // let cmd_str = cmd
    //     .cmd
    //     .replace("{magnet}", &item.magnet_link)
    //     .replace("{torrent}", &item.torrent_link)
    //     .replace("{title}", &item.title)
    //     .replace("{file}", &item.file_name);

    // let cmds = cmd.shell_cmd.split_whitespace().collect::<Vec<&str>>();
    // if let [base_cmd, args @ ..] = cmds.as_slice() {
    //     let cmd = Command::new(base_cmd)
    //         .args(args)
    //         .arg(&cmd_str)
    //         .stdin(Stdio::null())
    //         .stdout(Stdio::null())
    //         .stderr(Stdio::piped())
    //         .spawn();
    //
    //     let child = match cmd {
    //         Ok(child) => child,
    //         Err(e) => {
    //             return Err(format!("{}:\nFailed to run:\n{}", cmd_str, e).to_owned());
    //         }
    //     };
    //     let output = match child.wait_with_output() {
    //         Ok(output) => output,
    //         Err(e) => {
    //             return Err(format!("{}:\nFailed to get output:\n{}", cmd_str, e).to_owned());
    //         }
    //     };
    //
    //     if output.status.code() != Some(0) {
    //         let mut err = BufReader::new(&*output.stderr);
    //         let mut err_str = String::new();
    //         err.read_to_string(&mut err_str).unwrap_or(0);
    //         return Err(format!(
    //             "{}:\nExited with status code {}:\n{}",
    //             cmd_str, output.status, err_str
    //         )
    //         .to_owned());
    //     }
    //     Ok("Successfully ran command".to_owned())
    // } else {
    //     Err(format!(
    //         "Shell command is not properly formatted:\n{}",
    //         cmd.shell_cmd,
    //     )
    //     .to_owned())
    // }
}
