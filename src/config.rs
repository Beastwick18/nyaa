use std::path::PathBuf;

use crate::{
    app::{Context, Widgets, APP_NAME},
    client::{Client, ClientConfig},
    clip::ClipboardConfig,
    source::Sources,
    theme::{self, Theme},
    widget::{
        category::{self, ALL_CATEGORIES},
        filter::Filter,
        results::ColumnsConfig,
        sort::Sort,
    },
};
use confy::ConfyError;
use serde::{Deserialize, Serialize};

pub static CONFIG_FILE: &str = "config";

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Config {
    pub torrent_client_cmd: Option<String>,
    pub default_category: String,
    pub default_filter: Filter,
    pub default_sort: Sort,
    pub default_search: String,
    #[serde(alias = "default_theme")]
    pub theme: String,
    #[serde(alias = "default_source")]
    pub source: Sources,
    pub download_client: Client,
    pub date_format: String,
    pub base_url: String,
    pub request_proxy: Option<String>,
    pub timeout: u64,

    #[serde(rename = "clipboard")]
    pub clipboard: Option<ClipboardConfig>,
    #[serde(rename = "columns")]
    pub columns: Option<ColumnsConfig>,
    #[serde(rename = "client")]
    pub client: ClientConfig,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            torrent_client_cmd: None,
            default_category: ALL_CATEGORIES[0].entries[0].cfg.to_owned(),
            default_filter: Filter::NoFilter,
            default_sort: Sort::Date,
            source: Sources::NyaaHtml,
            download_client: Client::Cmd,
            theme: Theme::default().name,
            default_search: "".to_owned(),
            date_format: "%Y-%m-%d %H:%M".to_owned(),
            base_url: "https://nyaa.si/".to_owned(),
            request_proxy: None,
            timeout: 30,
            clipboard: None,
            columns: None,
            client: ClientConfig::default(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Config, ConfyError> {
        confy::load::<Config>(APP_NAME, CONFIG_FILE)
    }
    pub fn store(self) -> Result<(), ConfyError> {
        confy::store::<Config>(APP_NAME, CONFIG_FILE, self)
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
    pub fn apply(&self, ctx: &mut Context, w: &mut Widgets) {
        ctx.config = self.to_owned();
        w.search.input.input = ctx.config.default_search.to_owned();
        w.search.input.cursor = w.search.input.input.len();
        w.sort.selected.sort = ctx.config.default_sort.to_owned();
        w.filter.selected = ctx.config.default_filter.to_owned();
        ctx.client = ctx.config.download_client.to_owned();
        ctx.src = ctx.config.source.to_owned();

        // Load user-defined themes
        if let Err(e) = theme::load_user_themes(ctx) {
            ctx.show_error(e);
        }

        if let Some((i, _, theme)) = ctx.themes.get_full(&self.theme) {
            w.theme.selected = i;
            ctx.theme = theme.to_owned();
        }
        if let Some(ent) = category::find_category(ctx.config.default_category.to_owned()) {
            w.category.category = ent.id;
        }

        if let Err(e) = ctx.client.clone().load_config(ctx) {
            ctx.show_error(e);
        }
    }
}
