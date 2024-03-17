use std::{
    io::{BufReader, Read as _},
    process::{Command, Stdio},
};

use serde::{Deserialize, Serialize};

use crate::{app::App, source::Item};

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

            #[cfg(windows)]
            shell_cmd: "powershell -Command".to_owned(),
            #[cfg(unix)]
            shell_cmd: "sh -c".to_owned(),
        }
    }
}

pub fn load_config(app: &mut App) {
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

pub async fn download(item: &Item, app: &mut App) {
    load_config(app);
    let cmd = match app.config.client.cmd.to_owned() {
        Some(c) => c,
        None => {
            app.show_error("Failed to get cmd config");
            return;
        }
    };
    let cmd_str = cmd
        .cmd
        .replace("{magnet}", &item.magnet_link)
        .replace("{torrent}", &item.torrent_link)
        .replace("{title}", &item.title)
        .replace("{file}", &item.file_name);

    let cmds = cmd.shell_cmd.split_whitespace().collect::<Vec<&str>>();
    if let [base_cmd, args @ ..] = cmds.as_slice() {
        let cmd = Command::new(base_cmd)
            .args(args)
            .arg(&cmd_str)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn();

        let child = match cmd {
            Ok(child) => child,
            Err(e) => {
                app.show_error(format!("{}:\nFailed to run:\n{}", cmd_str, e));
                return;
            }
        };
        let output = match child.wait_with_output() {
            Ok(output) => output,
            Err(e) => {
                app.show_error(format!("{}:\nFailed to get output:\n{}", cmd_str, e));
                return;
            }
        };

        if output.status.code() != Some(0) {
            let mut err = BufReader::new(&*output.stderr);
            let mut err_str = String::new();
            err.read_to_string(&mut err_str).unwrap_or(0);
            app.show_error(format!(
                "{}:\nExited with status code {}:\n{}",
                cmd_str, output.status, err_str
            ));
        }
    } else {
        app.show_error(format!(
            "Shell command is not properly formatted:\n{}",
            cmd.shell_cmd,
        ))
    }
}
