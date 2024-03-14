use crate::{
    app::{App, Widgets, APP_NAME},
    client::{qbit::QbitConfig, Client},
    source::Sources,
    widget::{
        category::{self, ALL_CATEGORIES},
        filter::Filter,
        sort::Sort,
        theme::{self, THEMES},
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
    pub default_theme: String,
    pub default_search: String,
    pub default_source: Sources,
    pub default_client: Client,
    pub date_format: String,
    pub base_url: String,
    pub timeout: u64,

    #[serde(rename = "qBittorrent")]
    pub qbit: Option<QbitConfig>,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            torrent_client_cmd: None,
            default_category: ALL_CATEGORIES[0].entries[0].cfg.to_owned(),
            default_filter: Filter::NoFilter,
            default_sort: Sort::Date,
            default_source: Sources::NyaaHtml,
            default_client: Client::Cmd,
            default_theme: THEMES[0].name.to_owned(),
            default_search: "".to_owned(),
            date_format: "%Y-%m-%d %H:%M".to_owned(),
            base_url: "https://nyaa.si/".to_owned(),
            timeout: 30,
            qbit: None,
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
    pub fn apply(&self, app: &mut App, w: &mut Widgets) {
        app.config = self.to_owned();
        w.search.input.input = app.config.default_search.to_owned();
        w.search.input.cursor = w.search.input.input.len();
        w.sort.selected = app.config.default_sort.to_owned();
        w.filter.selected = app.config.default_filter.to_owned();
        app.client = app.config.default_client.to_owned();
        app.src = app.config.default_source.to_owned();
        if let Some((i, theme)) = theme::find_theme(app.config.default_theme.to_owned()) {
            w.theme.selected = i;
            app.theme = theme;
        }
        if let Some(ent) = category::find_category(app.config.default_category.to_owned()) {
            w.category.category = ent.id;
        }

        if let Err(e) = app.client.clone().load_config(app) {
            app.show_error(e);
        }
    }
}
