use color_eyre::Result;

use crate::{keys::KeyBindings, themes::Theme};

const DEFAULT_KEYBINDS_TOML: &str = include_str!("../.default-config/keybinds.toml");

pub struct AppConfig {}

pub struct Config {
    pub app: AppConfig,
    pub keys: KeyBindings,
    pub themes: Vec<Theme>,
}

impl Config {
    pub fn new() -> Result<Self> {
        Ok(Self {
            // TODO: load from disk
            app: AppConfig {},
            keys: toml::from_str(DEFAULT_KEYBINDS_TOML)?,
            themes: vec![],
        })
    }
}
