use std::{error::Error, path::PathBuf};

use crate::{
    app::{Context, Widgets, APP_NAME},
    client::{Client, ClientConfig},
    clip::ClipboardConfig,
    source::{SourceConfig, Sources},
    theme::{self, Theme},
};
use confy::ConfyError;
use serde::{Deserialize, Serialize};

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
    pub timeout: u64, // TODO: treat as "global" timeout, can overwrite per-source

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
            clipboard: None,
            client: ClientConfig::default(),
            sources: SourceConfig::default(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Config, ConfyError> {
        confy::load::<Config>(APP_NAME, CONFIG_FILE)
    }
    pub fn path() -> Result<PathBuf, ConfyError> {
        confy::get_configuration_file_path(APP_NAME, None).and_then(|p| {
            p.parent()
                .ok_or(ConfyError::BadConfigDirectory(
                    "Config directory does not have a parent folder".to_owned(),
                ))
                .map(|p| p.to_path_buf())
        })
    }
    pub fn apply(&self, ctx: &mut Context, w: &mut Widgets) -> Result<(), Box<dyn Error>> {
        ctx.config = self.clone();
        w.search.input.cursor = w.search.input.input.len();
        w.sort.selected.sort = 0;
        w.filter.selected = 0;
        ctx.client = ctx.config.download_client;
        ctx.src = ctx.config.source;
        ctx.src_info = ctx.src.info();

        // Load user-defined themes
        if let Some((i, _, theme)) = ctx.themes.get_full(&self.theme) {
            w.theme.selected = i;
            ctx.theme = theme.clone();
        }

        ctx.src.load_config(&mut ctx.config.sources);
        ctx.src.apply(ctx, w);

        ctx.client.load_config(ctx);
        theme::load_user_themes(ctx)?;

        // Load defaults for default source
        Ok(())
    }
}
