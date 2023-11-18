use crate::app::APP_NAME;
use confy::ConfyError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::nyaa::{Filter, Category, Sort};

pub static CONFIG_FILE: &str = "config";

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub torrent_client_cmd: String,
    pub default_category: Category,
    pub default_filter: Filter,
    pub default_sort: Sort,
}

impl Config {
    pub fn from_file() -> Result<Config, ConfyError> {
        confy::load::<Config>(APP_NAME, CONFIG_FILE)
    }

    pub fn get_path() -> PathBuf {
        confy::get_configuration_file_path(APP_NAME, CONFIG_FILE).unwrap()
    }
}

// fn get_download_dir() -> String {
//     if let Some(dir) = dirs::download_dir() {
//         if let Some(dir_s) = dir.to_str() {
//             return dir_s.to_owned()
//         }
//     }
//     "".to_owned()
// }

impl std::default::Default for Config {
    fn default() -> Config {
        Config {
            torrent_client_cmd: "bash -c 'curl {torrent} > \"{title}.torrent\"'".to_owned(),
            default_category: Category::AllAnime,
            default_filter: Filter::NoFilter,
            default_sort: Sort::Date,
        }
    }
}
