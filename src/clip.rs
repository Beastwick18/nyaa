use std::error::Error;

use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Selection {
    Primary,
    Clipboard,
    Both,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ClipboardConfig {
    pub cmd: Option<String>,
    pub shell_cmd: Option<String>,
    pub selection: Option<Selection>,
}

use clipboard::ClipboardProvider as _;

#[cfg(target_os = "linux")]
use {
    clipboard::x11_clipboard::{Clipboard, Primary, X11ClipboardContext},
    wl_clipboard_rs::copy::Error::WaylandConnection,
    wl_clipboard_rs::copy::{ClipboardType, MimeType, Options, Source},
};

use clipboard::ClipboardContext;

use crate::util::cmd::CommandBuilder;

pub fn copy_to_clipboard(
    link: String,
    conf: Option<ClipboardConfig>,
) -> Result<(), Box<dyn Error>> {
    if let Some(conf) = conf.to_owned() {
        if let Some(cmd) = conf.cmd {
            return CommandBuilder::new(cmd)
                .sub("{content}", &link)
                .run(conf.shell_cmd);
        }
    }

    #[cfg(target_os = "linux")]
    {
        let clip_type = match conf.as_ref().and_then(|sel| sel.selection) {
            Some(Selection::Primary) => ClipboardType::Primary,
            Some(Selection::Both) => ClipboardType::Both,
            None | Some(Selection::Clipboard) => ClipboardType::Regular,
        };
        let mut opts = Options::new();
        let res = opts.clipboard(clip_type).clone().copy(
            Source::Bytes(link.clone().into_bytes().into()),
            MimeType::Autodetect,
        );
        // If we successfully connected to wayland compositor, return result
        if !matches!(res, Err(WaylandConnection(_))) {
            return res.map_err(|e| e.into());
        }
        // Otherwise, try X11
        match conf.and_then(|sel| sel.selection) {
            Some(Selection::Primary) => X11ClipboardContext::<Primary>::new()
                .and_then(|mut s| s.set_contents(link))
                .map_err(|e| format!("Failed to copy to x11 \"primary\":\n{}", e).into()),
            Some(Selection::Clipboard) => X11ClipboardContext::<Clipboard>::new()
                .and_then(|mut s| s.set_contents(link))
                .map_err(|e| format!("Failed to copy to x11 \"clipboard\":\n{}", e).into()),
            Some(Selection::Both) => {
                let cb = X11ClipboardContext::<Clipboard>::new()
                    .and_then(|mut s| s.set_contents(link.clone()))
                    .map_err(|e| format!("Failed to copy to x11 \"clipboard\":\n{}", e));
                let pr = X11ClipboardContext::<Primary>::new()
                    .and_then(|mut s| s.set_contents(link))
                    .map_err(|e| format!("Failed to copy to x11 \"primary\":\n{}", e));
                let mut errors = String::new();
                if let Err(e) = cb {
                    errors.push_str(&e);
                }
                if let Err(e) = pr {
                    errors.push_str(&e);
                }
                if !errors.is_empty() {
                    return Err(errors.into());
                }
                Ok(())
            }
            None => ClipboardContext::new()
                .and_then(|mut s| s.set_contents(link))
                .map_err(|e| format!("Failed to copy to clipboard:\n{}", e).into()),
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        ClipboardContext::new()
            .and_then(|mut s| s.set_contents(link))
            .map_err(|e| format!("Failed to copy to clipboard:\n{}", e).into())
    }
}
