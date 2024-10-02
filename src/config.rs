use std::{
    error::Error,
    fs::{self, File, OpenOptions},
    io::{ErrorKind, Read, Write as _},
    path::{Path, PathBuf},
    str::FromStr,
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

pub static CONFIG_FILE: &str = "config.toml";

pub trait ConfigManager {
    fn load(&self) -> Result<Config, Box<dyn Error>>;
    fn store(&self, cfg: &Config) -> Result<(), Box<dyn Error>>;
    fn path(&self) -> PathBuf;
}

pub struct AppConfig {
    config_path: PathBuf,
}

impl AppConfig {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            config_path: get_configuration_folder(APP_NAME)?,
        })
    }
    pub fn from_path(config_path: String) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            config_path: PathBuf::from_str(&config_path)?,
        })
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Config {
    #[serde(alias = "default_theme")]
    pub theme: String,
    #[serde(rename = "default_source")]
    pub source: Sources,
    pub download_client: Client,
    pub date_format: Option<String>,
    pub relative_date: Option<bool>,
    pub relative_date_short: Option<bool>,
    pub request_proxy: Option<String>,
    pub timeout: u64,
    pub scroll_padding: usize,
    pub cursor_padding: usize,
    pub save_config_on_change: bool,
    pub hot_reload_config: bool,

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
            relative_date: None,
            relative_date_short: None,
            request_proxy: None,
            timeout: 30,
            scroll_padding: 3,
            cursor_padding: 4,
            save_config_on_change: true,
            hot_reload_config: true,

            notifications: None,
            clipboard: None,
            client: ClientConfig::default(),
            sources: SourceConfig::default(),
        }
    }
}

impl ConfigManager for AppConfig {
    fn load(&self) -> Result<Config, Box<dyn Error>> {
        load_path(self.config_path.join(CONFIG_FILE))
    }
    fn store(&self, cfg: &Config) -> Result<(), Box<dyn Error>> {
        store_path(self.config_path.join(CONFIG_FILE), cfg)
    }
    fn path(&self) -> PathBuf {
        self.config_path.clone()
    }
}

impl Config {
    pub fn full_apply(
        &self,
        path: PathBuf,
        ctx: &mut Context,
        w: &mut Widgets,
    ) -> Result<(), Box<dyn Error>> {
        // Load user-defined themes
        theme::load_user_themes(ctx, path)?;

        self.partial_apply(ctx, w)?;

        // Set download client
        ctx.client = ctx.config.download_client;
        // Set source
        ctx.src = ctx.config.source;
        // Set source info (categories, etc.)
        ctx.src_info = ctx.src.info();

        ctx.src.apply(ctx, w);
        if let Some(conf) = ctx.config.notifications {
            w.notification.load_config(&conf);
        }

        w.clients.table.select(ctx.client as usize);

        // Load defaults for default source
        Ok(())
    }

    pub fn partial_apply(&self, ctx: &mut Context, w: &mut Widgets) -> Result<(), Box<dyn Error>> {
        ctx.config = self.clone();

        // Set selected theme
        if let Some((i, _, theme)) = ctx.themes.get_full(&self.theme) {
            w.theme.selected = i;
            w.theme.table.select(i);
            ctx.theme = theme.clone();
        }

        // Load download client config
        ctx.client.load_config(&mut ctx.config.client);

        // Load current source config
        ctx.src.load_config(&mut ctx.config.sources);

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

fn store_path(path: impl AsRef<Path>, cfg: impl Serialize) -> Result<(), Box<dyn Error>> {
    let path = path.as_ref();
    let config_dir = path
        .parent()
        .ok_or(format!("{path:?} is a root or prefix"))?;
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
    let config_name: &str = Into::<Option<&'a str>>::into(config_name).unwrap_or("config");
    let path = get_configuration_folder(app_name)?.join(format!("{config_name}.toml"));
    Ok(path)
}

pub fn get_configuration_folder(app_name: &str) -> Result<PathBuf, Box<dyn Error>> {
    let project = ProjectDirs::from("rs", "", app_name)
        .ok_or("could not determine home directory path".to_string())?;

    let path = project.config_dir();
    let config_dir_str = path
        .to_str()
        .ok_or(format!("{path:?} is not valid Unicode"))?;

    Ok(config_dir_str.into())
}
