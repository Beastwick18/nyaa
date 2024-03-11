use std::{
    io::{BufReader, Read as _},
    process::{Command, Stdio},
};

use crate::{app::App, source::Item};

pub fn load_config(app: &mut App) {
    if app.config.torrent_client_cmd.is_none() {
        app.config.torrent_client_cmd = Some(
            #[cfg(windows)]
            "curl {torrent} -o ~\\Downloads\\{file}".to_owned(),
            #[cfg(unix)]
            "curl {torrent} > ~/{file}".to_owned(),
        );
    }
}

pub async fn download(item: &Item, app: &mut App) {
    let cmd = app.config.torrent_client_cmd.clone().unwrap_or_default();
    #[cfg(target_os = "windows")]
    let (cmd_str, cmd) = {
        let cmd_str = cmd
            .replace("{magnet}", &item.magnet_link)
            .replace("{torrent}", &item.torrent_link)
            .replace("{title}", &item.title)
            .replace("{file}", &item.file_name);

        let cmd = Command::new("powershell.exe")
            .arg("-Command")
            .arg(&cmd_str)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn();
        (cmd_str, cmd)
    };
    #[cfg(not(target_os = "windows"))]
    let (cmd_str, cmd) = {
        let cmd_str = cmd
            .replace("{magnet}", &shellwords::escape(&item.magnet_link))
            .replace("{torrent}", &shellwords::escape(&item.torrent_link))
            .replace("{title}", &shellwords::escape(&item.title))
            .replace("{file}", &shellwords::escape(&item.file_name));

        let cmd = Command::new("sh")
            .arg("-c")
            .arg(&cmd_str)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn();
        (cmd_str, cmd)
    };
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
}
