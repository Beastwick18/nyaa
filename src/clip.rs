use arboard::{Clipboard, GetExtLinux, LinuxClipboardKind, SetExtLinux};
use serde::{Deserialize, Serialize};

use crate::util::cmd::CommandBuilder;

#[derive(Default, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Selection {
    #[default]
    Clipboard,
    Primary,
    Secondary,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
}

impl Selection {
    fn get_kind(&self) -> LinuxClipboardKind {
        match self {
            Self::Primary => LinuxClipboardKind::Primary,
            Self::Clipboard => LinuxClipboardKind::Clipboard,
            Self::Secondary => LinuxClipboardKind::Secondary,
            //Self::Both => vec![LinuxClipboardKind::Secondary, LinuxClipboardKind::Primary],
        }
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct ClipboardConfig {
    pub cmd: Option<String>,
    pub shell_cmd: Option<String>,
    pub selection: Option<OneOrMany<Selection>>,
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
            if let Some(selection) = &config.selection {
                let x = match selection.to_owned() {
                    OneOrMany::One(one) => vec![one],
                    OneOrMany::Many(many) => many,
                };
                let errors = x
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
