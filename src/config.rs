use std::{
    error::Error,
    fs::{self, File, OpenOptions},
    io::{ErrorKind, Read, Write as _},
    path::{Path, PathBuf},
};

use crate::{
    app::{Context, Widgets, APP_NAME},
    client::{Client, ClientConfig},
    clip::ClipboardConfig,
    source::{SourceConfig, Sources},
    theme::{self, Theme},
    widget::notifications::NotificationConfig,
};
use directories::ProjectDirs;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub trait ConfigManager {
    fn load() -> Result<Config, Box<dyn Error>>;
    fn store(cfg: &Config) -> Result<(), Box<dyn Error>>;
    fn path() -> Result<PathBuf, Box<dyn Error>>;
}

pub struct AppConfig;

pub static CONFIG_FILE: &str = "config";

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Config {
    #[serde(alias = "default_theme")]
    pub theme: String,
    #[serde(rename = "default_source")]
    pub source: Sources,
    pub download_client: Client,
    pub date_format: Option<String>,
    pub request_proxy: Option<String>,
    pub timeout: u64,
    pub scroll_padding: usize,

    #[serde(rename = "notifications")]
    pub notifications: Option<NotificationConfig>,
    #[serde(rename = "clipboard")]
    pub clipboard: Option<ClipboardConfig>,
    #[serde(rename = "client")]
    pub client: ClientConfig,
    #[serde(rename = "source")]
    pub sources: SourceConfig,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            source: Sources::Nyaa,
            download_client: Client::Cmd,
            theme: Theme::default().name,
            date_format: None,
            request_proxy: None,
            timeout: 30,
            scroll_padding: 3,
            notifications: None,
            clipboard: None,
            client: ClientConfig::default(),
            sources: SourceConfig::default(),
        }
    }
}

impl ConfigManager for AppConfig {
    fn load() -> Result<Config, Box<dyn Error>> {
        get_configuration_file_path(APP_NAME, CONFIG_FILE).and_then(load_path)
    }
    fn store(cfg: &Config) -> Result<(), Box<dyn Error>> {
        get_configuration_file_path(APP_NAME, CONFIG_FILE).and_then(|p| store_path(p, cfg))
    }
    fn path() -> Result<PathBuf, Box<dyn Error>> {
        get_configuration_folder(APP_NAME)
    }
}

impl Config {
    pub fn apply<C: ConfigManager>(
        &self,
        ctx: &mut Context,
        w: &mut Widgets,
    ) -> Result<(), Box<dyn Error>> {
        ctx.config = self.clone();
        w.search.input.cursor = w.search.input.input.len();
        w.sort.selected.sort = 0;
        w.filter.selected = 0;
        ctx.client = ctx.config.download_client;
        ctx.src = ctx.config.source;
        ctx.src_info = ctx.src.info();

        ctx.src.load_config(&mut ctx.config.sources);
        ctx.src.apply(ctx, w);
        if let Some(conf) = ctx.config.notifications {
            w.notification.load_config(&conf);
        }

        ctx.client.load_config(ctx);
        let path = C::path().map_err(|e| e.to_string())?;
        // Load user-defined themes
        theme::load_user_themes(ctx, path)?;
        // Set selected theme
        if let Some((i, _, theme)) = ctx.themes.get_full(&self.theme) {
            w.theme.selected = i;
            w.theme.table.select(i);
            ctx.theme = theme.clone();
        }

        // Load defaults for default source
        Ok(())
    }
}

pub fn load_path<T: Serialize + DeserializeOwned + Default>(
    path: impl AsRef<Path>,
) -> Result<T, Box<dyn Error>> {
    let path = path.as_ref();
    match File::open(path) {
        Ok(mut cfg) => {
            let mut cfg_string = String::new();
            cfg.read_to_string(&mut cfg_string)
                .map_err(|e| format!("{path:?}\nUnable to read file:\n{e}"))?;

            let cfg_data = toml::from_str(&cfg_string);
            let data = cfg_data?;
            Ok(data)
        }
        Err(ref e) if e.kind() == ErrorKind::NotFound => {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let cfg = T::default();
            store_path(path, &cfg)?;
            Ok(cfg)
        }
        Err(e) => Err(e.into()),
    }
}

fn store_path<T: Serialize>(path: impl AsRef<Path>, cfg: T) -> Result<(), Box<dyn Error>> {
    let path = path.as_ref();
    let config_dir = path
        .parent()
        .ok_or_else(|| format!("{path:?} is a root or prefix"))?;
    fs::create_dir_all(config_dir)?;

    let s = toml::to_string_pretty(&cfg)?;

    let mut f = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;

    f.write_all(s.as_bytes())?;
    Ok(())
}

pub fn get_configuration_file_path<'a>(
    app_name: &str,
    config_name: impl Into<Option<&'a str>>,
) -> Result<PathBuf, Box<dyn Error>> {
    let config_name = config_name.into().unwrap_or("config");
    let path = get_configuration_folder(app_name)?.join(format!("{config_name}.toml"));
    Ok(path)
}

pub fn get_configuration_folder(app_name: &str) -> Result<PathBuf, Box<dyn Error>> {
    let project = ProjectDirs::from("rs", "", app_name)
        .ok_or_else(|| "could not determine home directory path".to_string())?;

    let path = project.config_dir();
    let config_dir_str = path
        .to_str()
        .ok_or_else(|| format!("{path:?} is not valid Unicode"))?;

    Ok(config_dir_str.into())
}
