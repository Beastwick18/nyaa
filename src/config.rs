use crate::{
    app::{App, Widgets, APP_NAME},
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
    pub torrent_client_cmd: String,
    pub default_category: String,
    pub default_filter: Filter,
    pub default_sort: Sort,
    pub default_theme: String,
    pub default_search: String,
    pub default_source: Sources,
    pub base_url: String,
    pub timeout: u64,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            #[cfg(windows)]
            torrent_client_cmd: "curl {torrent} > ~\\Downloads\\{file}".to_owned(),
            #[cfg(unix)]
            torrent_client_cmd: "curl {torrent} > ~/{file}".to_owned(),
            default_category: ALL_CATEGORIES[0].entries[0].cfg.to_owned(),
            default_filter: Filter::NoFilter,
            default_sort: Sort::Date,
            default_source: Sources::NyaaHtml,
            default_theme: THEMES[0].name.to_owned(),
            default_search: "".to_owned(),
            base_url: "https://nyaa.si/".to_owned(),
            timeout: 30,
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
        w.search.input.input = self.default_search.to_owned();
        w.search.input.cursor = w.search.input.input.len();
        w.sort.selected = self.default_sort.to_owned();
        w.filter.selected = self.default_filter.to_owned();
        app.src = self.default_source.to_owned();
        if let Some((i, theme)) = theme::find_theme(self.default_theme.to_lowercase()) {
            w.theme.selected = i;
            app.theme = theme;
        }
        if let Some(ent) = category::find_category(self.default_category.to_lowercase()) {
            w.category.category = ent.id;
        }
    }
}
