use std::error::Error;

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum X11Selection {
    Primary,
    Clipboard,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ClipboardConfig {
    pub cmd: Option<String>,
    pub shell_cmd: Option<String>,
    pub x11_selection: Option<X11Selection>,
}

#[cfg(target_os = "linux")]
use cli_clipboard::{
    linux_clipboard::LinuxClipboardContext,
    x11_clipboard::{Clipboard, Primary, X11ClipboardContext},
    ClipboardProvider as _,
};

#[cfg(not(target_os = "linux"))]
use cli_clipboard::{ClipboardContext, ClipboardProvider};

use crate::util::CommandBuilder;

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
        match conf.and_then(|sel| sel.x11_selection) {
            Some(X11Selection::Primary) => {
                return X11ClipboardContext::<Primary>::new()
                    .map_err(|e| format!("Failed to get context \"primary\" selection:\n{}", e))?
                    .set_contents(link)
                    .map_err(|e| format!("Failed to copy to clipboard:\n{}", e).into());
            }
            Some(X11Selection::Clipboard) => {
                return X11ClipboardContext::<Clipboard>::new()
                    .map_err(|e| format!("Failed to get context \"clipboard\" selection:\n{}", e))?
                    .set_contents(link)
                    .map_err(|e| format!("Failed to copy to clipboard:\n{}", e).into());
            }
            _ => {}
        }

        LinuxClipboardContext::new()
            .map_err(|e| format!("Failed to get linux clipboard context:\n{}", e))?
            .set_contents(link)
            .map_err(|e| format!("Failed to copy to clipboard:\n{}", e).into())
    }
    #[cfg(not(target_os = "linux"))]
    {
        ClipboardContext::new()
            .map_err(|e| format!("Failed to get clipboard context:\n{}", e))?
            .set_contents(link)
            .map_err(|e| format!("Failed to copy to clipboard:\n{}", e).into())
    }
}
