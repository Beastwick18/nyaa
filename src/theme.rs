use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use indexmap::IndexMap;
use ratatui::{prelude::Color, widgets::BorderType};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{app::Context, collection, config, source::SourceTheme, util::conv::color_to_tui};

pub static THEMES_PATH: &str = "themes";

#[derive(Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    #[serde(with = "color_to_tui")]
    pub bg: Color,
    #[serde(with = "color_to_tui")]
    pub fg: Color,
    #[serde(
        deserialize_with = "border_deserialize",
        serialize_with = "border_serialize"
    )]
    pub border: BorderType,
    #[serde(with = "color_to_tui")]
    pub border_color: Color,
    #[serde(with = "color_to_tui")]
    pub border_focused_color: Color,
    #[serde(with = "color_to_tui")]
    pub hl_bg: Color,
    #[serde(with = "color_to_tui")]
    pub solid_bg: Color,
    #[serde(with = "color_to_tui")]
    pub solid_fg: Color,
    #[serde(with = "color_to_tui", alias = "trusted")]
    pub success: Color,
    #[serde(with = "color_to_tui", alias = "remake")]
    pub error: Color,

    #[serde(default)]
    pub source: SourceTheme,
}

pub fn load_user_themes(ctx: &mut Context, config_path: PathBuf) -> Result<(), String> {
    let path = config_path.join(THEMES_PATH);
    if !path.exists() {
        return Ok(()); // Allow no theme folder
    }
    let path_str = path.to_owned();
    let path_str = path_str.to_string_lossy();
    if !path.is_dir() {
        return Err(format!("\"{}\" is not a directory", path_str));
    }
    let dir = match fs::read_dir(path) {
        Ok(d) => d,
        Err(e) => return Err(format!("Can't read directory \"{}\":{}\n", path_str, e)),
    };
    let themes = dir
        .filter_map(|f| {
            let f = match f {
                Ok(f) => f,
                Err(e) => {
                    ctx.show_error(format!("Failed to get theme file path :\n{}", e));
                    return None;
                }
            };
            let res = match Theme::from_path(f.path()) {
                Ok(t) => t,
                Err(e) => {
                    ctx.show_error(format!(
                        "Failed to parse theme \"{}\":\n{}",
                        f.file_name().to_string_lossy(),
                        e
                    ));
                    return None;
                }
            };
            Some((res.name.to_owned(), res))
        })
        .collect::<IndexMap<String, Theme>>();

    ctx.themes.extend(themes);
    Ok(())
}

pub fn border_serialize<S: Serializer>(
    border: &BorderType,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&match border {
        BorderType::Plain => "Plain".to_owned(),
        BorderType::Rounded => "Rounded".to_owned(),
        BorderType::Double => "Double".to_owned(),
        BorderType::Thick => "Thick".to_owned(),
        BorderType::QuadrantInside => "QuadrantInside".to_owned(),
        BorderType::QuadrantOutside => "QuadrantOutside".to_owned(),
    })
}

pub fn border_deserialize<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<BorderType, D::Error> {
    use serde::de::{Error, Unexpected};

    let border_string = String::deserialize(deserializer)?;

    Ok(match border_string.to_lowercase().as_str() {
        "plain" => BorderType::Plain,
        "rounded" => BorderType::Rounded,
        "double" => BorderType::Double,
        "thick" => BorderType::Thick,
        "quadrantoutside" => BorderType::QuadrantOutside,
        "quadrantinside" => BorderType::QuadrantInside,
        _ => {
            return Err(Error::invalid_value(
                Unexpected::Bytes(border_string.as_bytes()),
                &"border string",
            ));
        }
    })
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            name: "Default".to_owned(),
            bg: Color::Reset,
            fg: Color::White,
            border: BorderType::Plain,
            border_color: Color::Gray,
            border_focused_color: Color::LightCyan,
            hl_bg: Color::DarkGray,
            solid_bg: Color::White,
            solid_fg: Color::Black,
            success: Color::Green,
            error: Color::Red,
            source: Default::default(),
        }
    }
}

impl Theme {
    fn from_path(path: impl AsRef<Path>) -> Result<Theme, Box<dyn Error>> {
        config::load_path(path)
    }
}

pub fn default_themes() -> IndexMap<String, Theme> {
    collection![
        "Default".to_owned() => Theme::default(),
        "Dracula".to_owned() => Theme {
            name: "Dracula".to_owned(),
            bg: Color::Rgb(40, 42, 54),
            fg: Color::Rgb(248, 248, 242),
            border: BorderType::Rounded,
            border_color: Color::Rgb(98, 114, 164),
            border_focused_color: Color::Rgb(189, 147, 249),
            hl_bg: Color::Rgb(98, 114, 164),
            solid_fg: Color::Rgb(40, 42, 54),
            solid_bg: Color::Rgb(139, 233, 253),
            success: Color::Rgb(80, 250, 123),
            error: Color::Rgb(255, 85, 85),
            source: Default::default(),
        },
        "Gruvbox".to_owned() => Theme {
            name: "Gruvbox".to_owned(),
            bg: Color::Rgb(40, 40, 40),
            fg: Color::Rgb(235, 219, 178),
            border: BorderType::Plain,
            border_color: Color::Rgb(102, 92, 84),
            border_focused_color: Color::Rgb(214, 93, 14),
            hl_bg: Color::Rgb(80, 73, 69),
            solid_bg: Color::Rgb(69, 133, 136),
            solid_fg: Color::Rgb(235, 219, 178),
            success: Color::Rgb(152, 151, 26),
            error: Color::Rgb(204, 36, 29),
            source: Default::default(),
        },
        "Catppuccin Macchiato".to_owned() => Theme {
            name: "Catppuccin Macchiato".to_owned(),
            bg: Color::Rgb(24, 25, 38),
            fg: Color::Rgb(202, 211, 245),
            border: BorderType::Rounded,
            border_color: Color::Rgb(110, 115, 141),
            border_focused_color: Color::Rgb(125, 196, 228),
            hl_bg: Color::Rgb(110, 115, 141),
            solid_bg: Color::Rgb(166, 218, 149),
            solid_fg: Color::Rgb(24, 25, 38),
            success: Color::Rgb(166, 218, 149),
            error: Color::Rgb(237, 135, 150),
            source: Default::default(),
        },
    ]
}
