use std::path::{Path, PathBuf};

use color_eyre::Result;

use crate::{keys::KeyBindings, themes::Theme};

const DEFAULT_KEYBINDS_TOML: &str = include_str!("../.default-config/keybinds.toml");

pub struct AppConfig {}

impl AppConfig {
    fn new(_path: &Path) -> Self {
        // TODO: Load from path
        Self {}
    }
}

pub struct Config {
    pub path: PathBuf,
    pub app: AppConfig,
    pub keys: KeyBindings,
    pub themes: Vec<Theme>,
}

impl Config {
    pub fn new<P: Into<PathBuf>>(path: P) -> Result<Self> {
        let path = path.into();
        let app = AppConfig::new(&path);
        Ok(Self {
            path,
            app,
            // TODO: load from disk
            keys: toml::from_str(DEFAULT_KEYBINDS_TOML)?,
            themes: vec![],
        })
    }

    pub fn default_config_path() -> PathBuf {
        directories::BaseDirs::new()
            .unwrap()
            .config_dir()
            .to_path_buf()
    }
}
