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
use regex::{Regex, RegexBuilder};
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExcludeConfig {
    #[serde(default)]
    pub sentences: Vec<String>,
    #[serde(default)]
    pub regexes: Vec<String>,
    #[serde(default = "default_true")]
    pub case_insensitive: bool,
}

fn default_true() -> bool {
    true
}

impl Default for ExcludeConfig {
    fn default() -> Self {
        Self {
            sentences: vec![],
            regexes: vec![],
            case_insensitive: true,
        }
    }
}

impl ExcludeConfig {
    pub fn into_filter(self) -> ExcludeFilter {
        let mut compiled_regexes = Vec::new();

        for reg_str in self.regexes {
            let re_result = RegexBuilder::new(&reg_str)
                .case_insensitive(self.case_insensitive)
                .build();

            match re_result {
                Ok(re) => compiled_regexes.push(re),
                Err(e) => eprintln!("Failed to compile regex '{}': {}", reg_str, e),
            }
        }

        let processed_sentences = if self.case_insensitive {
            self.sentences
                .into_iter()
                .map(|s| s.to_lowercase())
                .collect()
        } else {
            self.sentences
        };

        ExcludeFilter {
            sentences: processed_sentences,
            compiled_regexes,
            case_insensitive: self.case_insensitive,
        }
    }
}

// To have regex compiled once
pub struct ExcludeFilter {
    sentences: Vec<String>,
    compiled_regexes: Vec<Regex>,
    case_insensitive: bool,
}

impl ExcludeFilter {
    pub fn should_exclude(&self, title: &str) -> bool {
        // Sentences optimization: lowercase only once
        let title_lower: String;
        let title_to_check = if self.case_insensitive {
            title_lower = title.to_lowercase();
            &title_lower
        } else {
            title
        };

        // fast substring match
        for sentence in &self.sentences {
            if title_to_check.contains(sentence) {
                return true;
            }
        }

        // pre compiled regex match
        // (use original title because the regex is already handling case)
        for re in &self.compiled_regexes {
            if re.is_match(title) {
                return true;
            }
        }

        false
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
    /// Tell if we yank all available magnet info or just the minimal magnet info when it is `false`:
    /// `magnet:?xt=urn:btih:691526c892951e9b41b7946524513f945e5c7c45&dn=Example.File.Name&tr=http://example.com/tracker/announce` become `magnet:?xt=urn:btih:691526c892951e9b41b7946524513f945e5c7c45` when `false`
    pub yank_full_magnet: bool,

    #[serde(rename = "notifications")]
    pub notifications: Option<NotificationConfig>,
    #[serde(rename = "clipboard")]
    pub clipboard: Option<ClipboardConfig>,
    #[serde(rename = "client")]
    pub client: ClientConfig,
    #[serde(rename = "source")]
    pub sources: SourceConfig,

    pub exclude: Option<ExcludeConfig>,
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
            yank_full_magnet: true,

            notifications: None,
            clipboard: None,
            client: ClientConfig::default(),
            sources: SourceConfig::default(),
            exclude: None,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn filter(sentences: &[&str], regexes: &[&str], case_insensitive: bool) -> ExcludeFilter {
        ExcludeConfig {
            sentences: sentences.iter().map(|s| s.to_string()).collect(),
            regexes: regexes.iter().map(|s| s.to_string()).collect(),
            case_insensitive,
        }
        .into_filter()
    }

    // --- Default ---

    #[test]
    fn default_case_insensitive_is_true() {
        assert!(ExcludeConfig::default().case_insensitive);
    }

    // --- Sentence matching ---

    #[test]
    fn sentence_case_sensitive() {
        let f = filter(&["bleach"], &[], false);
        assert!(f.should_exclude("bleach 720p"));
        assert!(!f.should_exclude("Bleach 720p"));
    }

    #[test]
    fn sentence_case_insensitive() {
        let f = filter(&["bleach"], &[], true);
        assert!(f.should_exclude("Bleach 720p"));
        assert!(f.should_exclude("BLEACH 720p"));
        assert!(f.should_exclude("bleach 720p"));
    }

    #[test]
    fn sentence_substring_match() {
        let f = filter(&["your forma"], &[], true);
        assert!(f.should_exclude("[Group] Your Forma - 01 [1080p]"));
        assert!(!f.should_exclude("[Group] Bleach - 01 [1080p]"));
    }

    #[test]
    fn multiple_sentences_any_match() {
        let f = filter(&["bleach", "naruto"], &[], true);
        assert!(f.should_exclude("Bleach - 01"));
        assert!(f.should_exclude("Naruto Shippuden - 01"));
        assert!(!f.should_exclude("One Piece - 01"));
    }

    #[test]
    fn no_filters_never_excludes() {
        let f = filter(&[], &[], true);
        assert!(!f.should_exclude("Anything Goes Here"));
    }

    // --- Regex matching ---

    #[test]
    fn regex_case_insensitive() {
        let f = filter(&[], &["^\\[SubGroup\\]"], true);
        assert!(f.should_exclude("[SubGroup] Show - 01"));
        assert!(f.should_exclude("[subgroup] Show - 01"));
        assert!(!f.should_exclude("Show - 01 [SubGroup]"));
    }

    #[test]
    fn regex_case_sensitive() {
        let f = filter(&[], &["^\\[SubGroup\\]"], false);
        assert!(f.should_exclude("[SubGroup] Show - 01"));
        assert!(!f.should_exclude("[subgroup] Show - 01"));
    }

    #[test]
    fn regex_invalid_does_not_panic() {
        // An invalid regex should be silently skipped, not crash.
        let f = filter(&[], &["[invalid regex"], true);
        assert!(!f.should_exclude("anything"));
    }

    #[test]
    fn sentences_and_regexes_combined() {
        let f = filter(&["bleach"], &["\\b480p\\b"], true);
        assert!(f.should_exclude("Bleach - 01 [1080p]")); // matched by sentence
        assert!(f.should_exclude("One Piece - 01 [480p]")); // matched by regex
        assert!(!f.should_exclude("One Piece - 01 [1080p]")); // no match
    }

    // --- TOML deserialization ---

    #[test]
    fn deserialize_minimal_toml() {
        let cfg: ExcludeConfig = toml::from_str(
            r#"
            sentences = ["bleach"]
        "#,
        )
        .unwrap();
        assert_eq!(cfg.sentences, ["bleach"]);
        assert!(cfg.regexes.is_empty());
        assert!(cfg.case_insensitive); // serde default = true
    }

    #[test]
    fn deserialize_full_toml() {
        let cfg: ExcludeConfig = toml::from_str(
            r#"
            sentences = ["bleach", "your forma"]
            regexes   = ["\\b480p\\b"]
            case_insensitive = false
        "#,
        )
        .unwrap();
        assert_eq!(cfg.sentences, ["bleach", "your forma"]);
        assert_eq!(cfg.regexes, ["\\b480p\\b"]);
        assert!(!cfg.case_insensitive);
    }
}
