use crate::{
    app::APP_NAME,
    source::Sources,
    widget::{category::ALL_CATEGORIES, filter::Filter, sort::Sort, theme::THEMES},
};
use confy::ConfyError;
use serde::{Deserialize, Serialize};

pub static CONFIG_FILE: &str = "config";

#[derive(Serialize, Deserialize)]
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
            torrent_client_cmd: "bash -c 'curl {torrent} > \"{title}.torrent\"'".to_owned(),
            default_category: ALL_CATEGORIES[0].entries[0].cfg.to_owned(),
            default_filter: Filter::NoFilter,
            default_sort: Sort::Date,
            default_source: Sources::NyaaHtml,
            default_theme: THEMES[0].name.to_owned(),
            default_search: "".to_owned(),
            base_url: "http://nyaa.si/".to_owned(),
            timeout: 30,
        }
    }
}

impl Config {
    pub fn from_file() -> Result<Config, ConfyError> {
        confy::load::<Config>(APP_NAME, CONFIG_FILE)
    }
}
