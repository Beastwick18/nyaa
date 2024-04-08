use std::error::Error;

use cli_clipboard::ClipboardProvider;

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
};

#[cfg(not(target_os = "linux"))]
use cli_clipboard::ClipboardContext;

use crate::util::CommandBuilder;

pub fn copy_to_clipboard(
    link: String,
    conf: Option<ClipboardConfig>,
) -> Result<(), Box<dyn Error>> {
    if let Some(conf) = conf.clone() {
        if let Some(cmd) = conf.cmd {
            // let shell = conf.shell_cmd.unwrap_or(CommandBuilder::default_shell());
            return match CommandBuilder::new(cmd)
                .sub("{content}", &link)
                .run(conf.shell_cmd)
            {
                Ok(_) => Ok(()),
                Err(e) => Err(e.into()),
            };
        }
    }

    #[cfg(target_os = "linux")]
    {
        let sel = conf
            .and_then(|sel| sel.x11_selection)
            .unwrap_or(X11Selection::Clipboard);
        if X11Selection::Clipboard == sel {
            if let Ok(ctx) = &mut X11ClipboardContext::<Clipboard>::new() {
                if let Err(e) = ctx.set_contents(link.clone()) {
                    return Err(format!("Failed to copy to \"clipboard\" selection:\n{}", e).into());
                }
                return Ok(());
            }
        } else if X11Selection::Primary == sel {
            if let Ok(ctx) = &mut X11ClipboardContext::<Primary>::new() {
                if let Err(e) = ctx.set_contents(link.clone()) {
                    return Err(format!("Failed to copy to \"primary\" selection:\n{}", e).into());
                }
                return Ok(());
            }
        }
        if let Ok(ctx) = &mut LinuxClipboardContext::new() {
            if let Err(e) = ctx.set_contents(link.clone()) {
                return Err(format!("Failed to copy to clipboard:\n{}", e).into());
            }
        } else {
            return Err("Failed to copy to linux clipboard".into());
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        let mut ctx: ClipboardContext = match ClipboardProvider::new() {
            Ok(ctx) => ctx,
            Err(e) => {
                return Err(format!("Failed to copy to clipboard:\n{}", e).into());
            }
        };
        if let Err(e) = ctx.set_contents(link.clone()) {
            return Err(format!("Failed to copy to clipboard:\n{}", e).into());
        }
    }
    Ok(())
}
