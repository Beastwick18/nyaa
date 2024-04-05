use std::{fs, path::PathBuf};

use confy::ConfyError;
use indexmap::IndexMap;
use ratatui::{style::Color, widgets::BorderType};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{app::Context, collection, config};

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
    #[serde(with = "color_to_tui")]
    pub trusted: Color,
    #[serde(with = "color_to_tui")]
    pub remake: Color,
    // pub warning: Color,
}

pub fn load_user_themes(ctx: &mut Context) -> Result<(), String> {
    let path = config::Config::path().map_err(|e| e.to_string())?;
    let path = path.join("themes");
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
    dir.for_each(|f| {
        let f = match f {
            Ok(f) => f,
            Err(e) => return ctx.show_error(format!("Failed to get theme file path :\n{}", e)),
        };
        let res = match Theme::from_path(f.path()) {
            Ok(t) => t,
            Err(e) => {
                return ctx.show_error(format!(
                    "Failed to parse theme \"{}\":\n{}",
                    f.file_name().to_string_lossy(),
                    e
                ))
            }
        };
        ctx.themes.insert(res.name.to_owned(), res);
    });
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
            border_color: Color::White,
            border_focused_color: Color::LightCyan,
            hl_bg: Color::DarkGray,
            solid_bg: Color::White,
            solid_fg: Color::Black,
            trusted: Color::Green,
            remake: Color::Red,
        }
    }
}

impl Theme {
    fn from_path(path: PathBuf) -> Result<Theme, ConfyError> {
        confy::load_path::<Theme>(path)
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
            trusted: Color::Rgb(80, 250, 123),
            remake: Color::Rgb(255, 85, 85),
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
            trusted: Color::Rgb(152, 151, 26),
            remake: Color::Rgb(204, 36, 29),
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
            trusted: Color::Rgb(166, 218, 149),
            remake: Color::Rgb(237, 135, 150),
        },
    ]
}
