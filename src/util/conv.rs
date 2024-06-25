use std::error::Error;

use crossterm::event::{KeyCode, KeyModifiers, MediaKeyCode, ModifierKeyCode};
use reqwest::Url;

pub fn get_hash(magnet: String) -> Option<String> {
    magnet
        .split_once("xt=urn:btih:")
        .and_then(|m| m.1.split_once('&').map(|m| m.0.to_owned()))
}

pub fn add_protocol<S: Into<String>>(
    url: S,
    default_https: bool,
) -> Result<Url, Box<dyn Error + Send + Sync>> {
    let url = url.into();
    if let Some((method, other)) = url.split_once(':') {
        if matches!(method, "http" | "https" | "socks5") && matches!(other.get(..2), Some("//")) {
            return Ok(url.parse::<Url>()?);
        }
    }
    let protocol = match default_https {
        true => "https",
        false => "http",
    };
    Ok(format!("{}://{}", protocol, url).parse::<Url>()?)
}

pub fn to_bytes(size: &str) -> usize {
    let mut split = size.split_whitespace();
    let f = split
        .next()
        .and_then(|b| b.parse::<f64>().ok())
        .unwrap_or(-1.);
    let power = match split.last().and_then(|u| u.chars().next()) {
        Some('T') => 4,
        Some('G') => 3,
        Some('M') => 2,
        Some('K') => 1,
        _ => 0,
    };
    (1024_f64.powi(power) * f) as usize
}

pub fn shorten_number(n: u32) -> String {
    if n >= 10000 {
        format!("{}K", n / 1000)
    } else {
        n.to_string()
    }
}

pub fn key_to_string(key: KeyCode, modifier: KeyModifiers) -> String {
    let key = match key {
        KeyCode::Backspace => "BS".to_owned(),
        KeyCode::Enter => "CR".to_owned(),
        KeyCode::Left => "Left".to_owned(),
        KeyCode::Right => "Right".to_owned(),
        KeyCode::Up => "Up".to_owned(),
        KeyCode::Down => "Down".to_owned(),
        KeyCode::Home => "Home".to_owned(),
        KeyCode::End => "End".to_owned(),
        KeyCode::PageUp => "PgUp".to_owned(),
        KeyCode::PageDown => "PgDown".to_owned(),
        KeyCode::Tab | KeyCode::BackTab => "Tab".to_owned(),
        KeyCode::Delete => "Del".to_owned(),
        KeyCode::Insert => "Ins".to_owned(),
        KeyCode::F(f) => format!("F{}", f),
        KeyCode::Char(' ') => "Space".to_owned(),
        KeyCode::Char(c) => match modifier {
            KeyModifiers::NONE | KeyModifiers::SHIFT => return c.to_string(),
            _ => c.to_string(),
        },
        KeyCode::Esc => "Esc".to_owned(),
        KeyCode::Null => "Null".to_owned(),
        KeyCode::CapsLock => "CapsLock".to_owned(),
        KeyCode::ScrollLock => "ScrollLock".to_owned(),
        KeyCode::NumLock => "NumLock".to_owned(),
        KeyCode::PrintScreen => "Print".to_owned(),
        KeyCode::Pause => "Pause".to_owned(),
        KeyCode::Menu => "Menu".to_owned(),
        KeyCode::KeypadBegin => "Begin".to_owned(),
        KeyCode::Media(m) => match m {
            MediaKeyCode::Play => "MediaPlay",
            MediaKeyCode::Pause => "MediaPause",
            MediaKeyCode::PlayPause => "MediaPlayPause",
            MediaKeyCode::Reverse => "MediaReverse",
            MediaKeyCode::Stop => "MediaStop",
            MediaKeyCode::FastForward => "MediaFastForward",
            MediaKeyCode::Rewind => "MediaRewind",
            MediaKeyCode::TrackNext => "MediaTrackNext",
            MediaKeyCode::TrackPrevious => "MediaTrackPrevious",
            MediaKeyCode::Record => "MediaRecord",
            MediaKeyCode::LowerVolume => "MediaLowerVolume",
            MediaKeyCode::RaiseVolume => "MediaRaiseVolume",
            MediaKeyCode::MuteVolume => "MediaMuteVolume",
        }
        .to_owned(),
        KeyCode::Modifier(m) => match m {
            ModifierKeyCode::LeftShift => "LeftShift",
            ModifierKeyCode::LeftControl => "LeftControl",
            ModifierKeyCode::LeftAlt => "LeftAlt",
            ModifierKeyCode::LeftSuper => "LeftSuper",
            ModifierKeyCode::LeftHyper => "LeftHyper",
            ModifierKeyCode::LeftMeta => "LeftMeta",
            ModifierKeyCode::RightShift => "RightShift",
            ModifierKeyCode::RightControl => "RightControl",
            ModifierKeyCode::RightAlt => "RightAlt",
            ModifierKeyCode::RightSuper => "RightSuper",
            ModifierKeyCode::RightHyper => "RightHyper",
            ModifierKeyCode::RightMeta => "RightMeta",
            ModifierKeyCode::IsoLevel3Shift => "IsoLevel3Shift",
            ModifierKeyCode::IsoLevel5Shift => "IsoLevel5Shift",
        }
        .to_owned(),
    };
    let modifier = match modifier {
        KeyModifiers::CONTROL => "C-",
        KeyModifiers::SHIFT => "S-",
        KeyModifiers::ALT => "A-",
        KeyModifiers::SUPER => "U-",
        KeyModifiers::META => "M-",
        KeyModifiers::HYPER => "H-",
        _ => "",
    };
    format!("<{}{}>", modifier, key)
}

// From https://github.com/uttarayan21/color-to-tui
pub mod color_to_tui {
    use ratatui::style::Color;
    use serde::{Deserialize as _, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(color: &Color, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&match color {
            Color::Reset => "Reset".to_string(),
            Color::Red => "Red".to_string(),
            Color::Green => "Green".to_string(),
            Color::Black => "Black".to_string(),
            Color::Yellow => "Yellow".to_string(),
            Color::Blue => "Blue".to_string(),
            Color::Magenta => "Magenta".to_string(),
            Color::Cyan => "Cyan".to_string(),
            Color::Gray => "Gray".to_string(),
            Color::White => "White".to_string(),

            Color::DarkGray => "DarkGray".to_string(),
            Color::LightBlue => "LightBlue".to_string(),
            Color::LightCyan => "LightCyan".to_string(),
            Color::LightGreen => "LightGreen".to_string(),
            Color::LightMagenta => "LightMagenta".to_string(),
            Color::LightRed => "LightRed".to_string(),
            Color::LightYellow => "LightYellow".to_string(),
            Color::Indexed(index) => format!("{:03}", index),
            Color::Rgb(r, g, b) => format!("#{:02X}{:02X}{:02X}", r, g, b),
        })
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Color, D::Error> {
        use serde::de::{Error, Unexpected};

        let color_string = String::deserialize(deserializer)?;
        Ok(match color_string.to_lowercase().as_str() {
            "reset" => Color::Reset,
            "red" => Color::Red,
            "green" => Color::Green,
            "black" => Color::Black,
            "yellow" => Color::Yellow,
            "blue" => Color::Blue,
            "magenta" => Color::Magenta,
            "cyan" => Color::Cyan,
            "gray" => Color::Gray,
            "white" => Color::White,

            "darkgray" => Color::DarkGray,
            "lightblue" => Color::LightBlue,
            "lightcyan" => Color::LightCyan,
            "lightgreen" => Color::LightGreen,
            "lightmagenta" => Color::LightMagenta,
            "lightred" => Color::LightRed,
            "lightyellow" => Color::LightYellow,
            _ => match color_string.len() {
                3 => {
                    let index = color_string.parse::<u8>();
                    if let Ok(index) = index {
                        Color::Indexed(index)
                    } else {
                        return Err(Error::invalid_type(
                            Unexpected::Bytes(color_string.as_bytes()),
                            &"u8 index color",
                        ));
                    }
                }
                4 | 7 => {
                    if !color_string.starts_with('#') {
                        return Err(Error::invalid_value(
                            Unexpected::Char(color_string.chars().next().unwrap()),
                            &"# at the start",
                        ));
                    }

                    let color_string = color_string.trim_start_matches('#');

                    let (r, g, b);

                    match color_string.len() {
                        6 => {
                            r = u8::from_str_radix(&color_string[0..2], 16);
                            g = u8::from_str_radix(&color_string[2..4], 16);
                            b = u8::from_str_radix(&color_string[4..6], 16);
                        }
                        3 => {
                            r = u8::from_str_radix(&color_string[0..1], 16).map(|r| r * 17);
                            g = u8::from_str_radix(&color_string[1..2], 16).map(|g| g * 17);
                            b = u8::from_str_radix(&color_string[2..3], 16).map(|b| b * 17);
                        }
                        _ => unreachable!("Can't be reached since already checked"),
                    }

                    match (r, g, b) {
                        (Ok(r), Ok(g), Ok(b)) => Color::Rgb(r, g, b),
                        (_, _, _) => {
                            return Err(Error::invalid_value(
                                Unexpected::Bytes(color_string.as_bytes()),
                                &"hex color string",
                            ));
                        }
                    }
                }
                _ => {
                    return Err(serde::de::Error::invalid_length(
                        color_string.len(),
                        &"color string with length 4 or 7",
                    ))
                }
            },
        })
    }
}
