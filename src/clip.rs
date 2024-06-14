use arboard::{Clipboard, GetExtLinux, LinuxClipboardKind, SetExtLinux};
use serde::{Deserialize, Serialize};

use crate::util::cmd::CommandBuilder;

#[derive(Default, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Selection {
    #[default]
    Clipboard,
    Primary,
    Secondary,
    Both,
}

impl Selection {
    fn get_all(self) -> Vec<LinuxClipboardKind> {
        match self {
            Self::Primary => vec![LinuxClipboardKind::Primary],
            Self::Clipboard => vec![LinuxClipboardKind::Clipboard],
            Self::Secondary => vec![LinuxClipboardKind::Secondary],
            Self::Both => vec![LinuxClipboardKind::Secondary, LinuxClipboardKind::Primary],
        }
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct ClipboardConfig {
    pub cmd: Option<String>,
    pub shell_cmd: Option<String>,
    pub selection: Option<Selection>,
}

pub struct ClipboardManager {
    clipboard: Option<Clipboard>,
    config: ClipboardConfig,
}

impl ClipboardManager {
    pub fn new(conf: ClipboardConfig) -> (ClipboardManager, Option<String>) {
        // Dont worry about connecting to OS clipboard if using command
        if conf.cmd.is_some() {
            return (
                Self {
                    clipboard: None,
                    config: conf,
                },
                None,
            );
        }

        let clipboard = Clipboard::new();
        let err = clipboard.as_ref().err().map(|x| x.to_string());
        let cb = clipboard.ok();
        (
            Self {
                clipboard: cb,
                config: conf,
            },
            err,
        )
    }

    pub fn try_copy(&mut self, content: &String) -> Result<(), String> {
        if let Some(cmd) = self.config.cmd.clone() {
            return CommandBuilder::new(cmd)
                .sub("{content}", content)
                .run(self.config.shell_cmd.clone())
                .map_err(|e| e.to_string());
        }
        match &mut self.clipboard {
            // Some(cb) => Ok(cb.set_text(content)?),
            Some(cb) => Self::copy(&self.config, cb, content).map_err(|e| e.to_string()),
            None => Err("The clipboard is not loaded".to_owned()),
        }
    }

    fn copy(
        config: &ClipboardConfig,
        clipboard: &mut Clipboard,
        content: &String,
    ) -> Result<(), String> {
        #[cfg(target_os = "linux")]
        {
            let errors = config
                .selection
                .unwrap_or_default()
                .get_all()
                .into_iter()
                .filter_map(|t| {
                    let res = clipboard.set().clipboard(t).text(content);
                    let _ = clipboard.get().clipboard(t).text();
                    res.err()
                        .map(|e| format!("Failed to copy to \"{t:?}\" selection:\n{e}"))
                })
                .collect::<Vec<String>>();
            if !errors.is_empty() {
                return Err(errors.join("\n\n"));
            }
            Ok(())
        }
        #[cfg(not(target_os = "linux"))]
        {
            clipboard
                .set_text(content)
                .map_err(|e| format!("Failed to copy:\n{e}"))?;
            let _ = clipboard.get_text();
            Ok(())
        }
    }
}
