use arboard::{Clipboard, GetExtLinux, LinuxClipboardKind, SetExtLinux};
use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, OneOrMany};

use crate::util::cmd::CommandBuilder;

#[derive(Default, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Selection {
    #[default]
    Clipboard,
    Primary,
    Secondary,
}

impl Selection {
    fn get_kind(&self) -> LinuxClipboardKind {
        match self {
            Self::Primary => LinuxClipboardKind::Primary,
            Self::Clipboard => LinuxClipboardKind::Clipboard,
            Self::Secondary => LinuxClipboardKind::Secondary,
        }
    }
}

#[serde_as]
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct ClipboardConfig {
    pub cmd: Option<String>,
    pub shell_cmd: Option<String>,
    pub osc52: bool,
    #[serde_as(as = "Option<OneOrMany<_>>")]
    #[serde(default)]
    pub selection: Option<Vec<Selection>>,
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
        if self.config.osc52 {
            print!(
                "\x1B]52;c;{}\x07",
                base64::engine::general_purpose::STANDARD.encode(content)
            );

            return Ok(());
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
            if let Some(selection) = &config.selection {
                let errors = selection
                    .iter()
                    .map(Selection::get_kind)
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
                return Ok(());
            }
        }
        clipboard
            .set_text(content)
            .map_err(|e| format!("Failed to copy:\n{e}"))?;
        let _ = clipboard.get_text();
        Ok(())
    }
}
