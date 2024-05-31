use crossterm::event::{KeyCode, KeyModifiers, MediaKeyCode, ModifierKeyCode};
use regex::Regex;

pub fn add_protocol<S: Into<String>>(url: S, default_https: bool) -> String {
    let protocol = match default_https {
        true => "https",
        false => "http",
    };
    let url = url.into();
    let re = Regex::new(r"^(https?|socks5)?://.+$").unwrap();
    match re.is_match(&url) {
        true => url,
        // Assume http(s) if not present
        false => format!("{}://{}", protocol, url),
    }
}

pub fn to_bytes(size: &str) -> usize {
    let mut split = size.split_whitespace();
    let b = split.next().unwrap_or("0");
    let unit = split.last().unwrap_or("B");
    let f = b.parse::<f64>().unwrap_or(0.0);
    let power = match unit.chars().next().unwrap_or('B') {
        'T' => 4,
        'G' => 3,
        'M' => 2,
        'K' => 1,
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
