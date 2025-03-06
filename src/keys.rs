use std::fmt::Display;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use derive_more::{Deref, DerefMut};
use enum_assoc::Assoc;
use indexmap::IndexMap;
use ratatui::style::Color;
use serde::{Deserialize, Deserializer};

use crate::{action::UserAction, app::Mode};

// Keys that cannot be used in combos, and will cancel the current combo.
// These keys may be used by themselves, as a single key keybind
pub static NON_COMBO: &[KeyCode] = &[KeyCode::Esc];

/// KeyBindings are a collection of *key*-*user action* pairs, seperated by mode
#[derive(Clone, Debug, Default, Deref, DerefMut)]
pub struct KeyBindings(pub IndexMap<Mode, IndexMap<Vec<KeyEvent>, OneOrManyActions>>);

#[derive(Default, Clone)]
pub struct KeyCombo {
    status: KeyComboStatus,
    repeat: Option<u8>,
    events: Vec<KeyEvent>,
}

impl KeyCombo {
    pub fn status(&self) -> &KeyComboStatus {
        &self.status
    }

    pub fn set_status(&mut self, status: KeyComboStatus) {
        self.status = status;
    }

    pub fn repeat(&self) -> &Option<u8> {
        &self.repeat
    }

    fn push_digit(&mut self, digit: u8) {
        let digit = digit.min(9);
        let result = self
            .repeat
            .map(|r| r.saturating_mul(10).saturating_add(digit))
            .unwrap_or(digit);
        self.repeat = Some(result);
    }

    pub fn events(&self) -> &Vec<KeyEvent> {
        &self.events
    }

    pub fn clear(&mut self) {
        self.repeat = None;
        self.events.clear();
    }

    pub fn push_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char(c) if c.is_ascii_digit() => {
                self.push_digit((c as u8).saturating_sub(b'0'))
            }
            _ => self.events.push(key),
        }
    }
}

#[derive(Assoc, Clone, Copy, Default, PartialEq, Eq)]
#[func(pub const fn color(&self) -> Color)]
pub enum KeyComboStatus {
    #[assoc(color = Color::White)]
    Pending,
    #[assoc(color = Color::Cyan)]
    Successful,
    #[default]
    #[assoc(color = Color::DarkGray)]
    Cancelled,
    #[assoc(color = Color::Red)]
    Unmatched,
}

#[derive(Assoc, Clone, Debug, Deserialize)]
#[func(pub fn multiplier(&self) -> u8)]
#[func(pub fn actions(&self) -> Vec<UserAction>)]
#[serde(untagged)]
pub enum OneOrManyActions {
    #[assoc(multiplier = 1, actions = vec![_0.clone()])]
    One(UserAction),
    #[assoc(multiplier = 1, actions = _0.clone())]
    Many(Vec<UserAction>),
    #[assoc(multiplier = *_0, actions = vec![_1.clone()])]
    Repeat(u8, UserAction),
}

impl Display for OneOrManyActions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OneOrManyActions::One(user_action) => write!(f, "{:?}", user_action),
            OneOrManyActions::Many(user_actions) => {
                write!(
                    f,
                    "{}",
                    user_actions
                        .iter()
                        .map(|a| format!("{:?}", a))
                        .collect::<Vec<String>>()
                        .join(", ")
                )
            }
            OneOrManyActions::Repeat(n, user_action) => write!(f, "{n}×{:?}", user_action),
        }
    }
}

// #[derive(Clone, Debug, Deserialize)]
// #[serde(untagged)]
// pub enum UserActionWrapped {
//     Unit(UserAction),
//     // Other(String), // TODO: Parse custom syntax, like "Action(arg1, ...)"
// }

impl<'de> Deserialize<'de> for KeyBindings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let parsed_map =
            IndexMap::<Mode, IndexMap<String, OneOrManyActions>>::deserialize(deserializer)?;

        let keybindings = parsed_map
            .into_iter()
            .map(|(mode, inner_map)| {
                let converted_inner_map = inner_map
                    .into_iter()
                    .map(|(key_str, cmd)| (parse_key_sequence(&key_str).unwrap(), cmd))
                    .collect();
                (mode, converted_inner_map)
            })
            .collect();

        Ok(KeyBindings(keybindings))
    }
}

// fn parse_wrapped_actions(cmd: UserActionWrapped) -> Result<UserAction, String> {
//     Ok(match cmd {
//         UserActionWrapped::Unit(unit) => unit,
//         // UserActionWrapped::Other(other) => parse_other_action_simple(other)?,
//     })
// }

// fn parse_other_action_simple(other: String) -> Result<UserAction, String> {
//     let tokens = other.split_ascii_whitespace().collect::<Vec<&str>>();
//     match tokens.as_slice() {
//         [key, val] => {
//             let mut table = toml::Table::new();
//             table.insert(key.to_string(), toml::Value::String(val.to_string()));
//             Ok(table.try_into().unwrap())
//         }
//         _ => Err("UserAction is not properly formatted".to_string()),
//     }
// }

// // TODO: More complex case
// fn parse_other_action(other: String) -> UserAction {
//     // TODO: more complex parsing, with handling of quotes
//     let mut tokens = other
//         .split_whitespace()
//         .map(ToString::to_string)
//         .collect::<Vec<String>>();
//
//     let val = tokens.pop().unwrap(); // TODO: handle invalid cases
//     let key = tokens.pop().unwrap();
//
//     let mut initial_table = toml::Table::new();
//     initial_table.insert(key, toml::Value::String(val));
//     let mut table: toml::Value = toml::Value::Table(initial_table);
//
//     while let Some(new_key) = tokens.pop() {
//         let mut new_table = toml::Table::new();
//         new_table.insert(new_key, table.clone());
//         table = toml::Value::Table(new_table);
//     }
//
//     match table {
//         toml::Value::Table(m) => m.try_into().unwrap(),
//         _ => panic!("ermmmm"),
//     }
// }

fn parse_key_event(raw: &str) -> Result<KeyEvent, String> {
    // let raw_lower = raw.to_ascii_lowercase();
    let (remaining, modifiers) = extract_modifiers(raw);
    parse_key_code_with_modifiers(remaining, modifiers)
}

fn extract_modifiers(raw: &str) -> (&str, KeyModifiers) {
    let mut modifiers = KeyModifiers::empty();
    let mut current = raw;
    // let mut lower = lower_string.as_str();

    loop {
        match (current.to_ascii_lowercase(), current) {
            (rest_lower, rest_upper) if rest_lower.starts_with("ctrl-") => {
                modifiers.insert(KeyModifiers::CONTROL);
                current = &rest_upper[5..];
                // lower = &rest_lower[5..];
            }
            (rest, rest_upper) if rest.starts_with("alt-") => {
                modifiers.insert(KeyModifiers::ALT);
                current = &rest_upper[4..];
                // lower = &rest[4..];
            }
            (rest, rest_upper) if rest.starts_with("shift-") => {
                modifiers.insert(KeyModifiers::SHIFT);
                current = &rest_upper[6..];
                // lower = &rest[6..];
            }
            // Shorthand versions
            (rest, rest_upper) if rest.starts_with("c-") => {
                modifiers.insert(KeyModifiers::CONTROL);
                current = &rest_upper[2..];
                // lower = &rest[2..];
            }
            (rest, rest_upper) if rest.starts_with("a-") => {
                modifiers.insert(KeyModifiers::ALT);
                current = &rest_upper[2..];
                // lower = &rest[2..];
            }
            (rest, rest_upper) if rest.starts_with("s-") => {
                modifiers.insert(KeyModifiers::SHIFT);
                current = &rest_upper[2..];
                // lower = &rest[2..];
            }
            _ => break, // break out of the loop if no known prefix is detected
        };
    }

    (current, modifiers)
}

fn parse_key_code_with_modifiers(
    raw: &str,
    mut modifiers: KeyModifiers,
) -> Result<KeyEvent, String> {
    let lower = raw.to_ascii_lowercase();
    let c = match lower.as_str() {
        "esc" => KeyCode::Esc,
        "enter" => KeyCode::Enter,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "home" => KeyCode::Home,
        "end" => KeyCode::End,
        "pageup" => KeyCode::PageUp,
        "pagedown" => KeyCode::PageDown,
        // NOTE: S-tab is equivalent
        "backtab" => {
            modifiers.insert(KeyModifiers::SHIFT);
            KeyCode::BackTab
        }
        "backspace" => KeyCode::Backspace,
        "delete" => KeyCode::Delete,
        "insert" => KeyCode::Insert,
        "f1" => KeyCode::F(1),
        "f2" => KeyCode::F(2),
        "f3" => KeyCode::F(3),
        "f4" => KeyCode::F(4),
        "f5" => KeyCode::F(5),
        "f6" => KeyCode::F(6),
        "f7" => KeyCode::F(7),
        "f8" => KeyCode::F(8),
        "f9" => KeyCode::F(9),
        "f10" => KeyCode::F(10),
        "f11" => KeyCode::F(11),
        "f12" => KeyCode::F(12),
        "space" => KeyCode::Char(' '),
        "hyphen" => KeyCode::Char('-'),
        "minus" => KeyCode::Char('-'),
        "lt" => KeyCode::Char('<'),
        "gt" => KeyCode::Char('>'),
        "tab" => {
            if modifiers.contains(KeyModifiers::SHIFT) {
                KeyCode::BackTab
            } else {
                KeyCode::Tab
            }
        }
        c if c.len() == 1 => {
            let mut c = raw.chars().next().unwrap();
            if c.is_ascii_uppercase() {
                modifiers.insert(KeyModifiers::SHIFT)
            } else if modifiers.contains(KeyModifiers::SHIFT) {
                c = c.to_ascii_uppercase();
            }
            KeyCode::Char(c)
        }
        _ => return Err(format!("Unable to parse {raw}")),
    };
    Ok(KeyEvent::new(c, modifiers))
}

pub fn parse_key_sequence(raw: &str) -> Result<Vec<KeyEvent>, String> {
    let mut inside = false;
    let mut keys: Vec<KeyEvent> = Vec::new();
    let mut working: String = String::new();
    for c in raw.chars() {
        if c == '>' && inside {
            keys.push(parse_key_event(&working)?);
            working.clear();
            inside = false;
        } else if c == '<' {
            inside = true;
        } else if inside {
            working.push(c);
        } else {
            keys.push(parse_key_event(&c.to_string())?);
        }
    }
    Ok(keys)
}

// pub fn parse_key_sequence(raw: &str) -> Result<Vec<KeyEvent>, String> {
//     if raw.chars().filter(|c| *c == '>').count() != raw.chars().filter(|c| *c == '<').count() {
//         return Err(format!("Unable to parse `{}`", raw));
//     }
//     let raw = if !raw.contains("><") {
//         let raw = raw.strip_prefix('<').unwrap_or(raw);
//         let raw = raw.strip_prefix('>').unwrap_or(raw);
//         raw
//     } else {
//         raw
//     };
//     let sequences = raw
//         .split("><")
//         .map(|seq| {
//             if let Some(s) = seq.strip_prefix('<') {
//                 s
//             } else if let Some(s) = seq.strip_suffix('>') {
//                 s
//             } else {
//                 seq
//             }
//         })
//         .collect::<Vec<_>>();
//
//     sequences.into_iter().map(parse_key_event).collect()
// }

pub fn key_event_to_string(key_event: &KeyEvent) -> String {
    let char;
    let key_code = match key_event.code {
        KeyCode::Backspace => "backspace",
        KeyCode::Enter => "enter",
        KeyCode::Left => "left",
        KeyCode::Right => "right",
        KeyCode::Up => "up",
        KeyCode::Down => "down",
        KeyCode::Home => "home",
        KeyCode::End => "end",
        KeyCode::PageUp => "pageup",
        KeyCode::PageDown => "pagedown",
        KeyCode::Tab => "tab",
        KeyCode::BackTab => "tab",
        KeyCode::Delete => "delete",
        KeyCode::Insert => "insert",
        KeyCode::F(c) => {
            char = format!("f{c}");
            &char
        }
        KeyCode::Char(' ') => "space",
        KeyCode::Char(c) => {
            char = c.to_string();
            &char
        }
        KeyCode::Esc => "esc",
        KeyCode::Null => "",
        KeyCode::CapsLock => "",
        KeyCode::Menu => "",
        KeyCode::ScrollLock => "",
        KeyCode::Media(_) => "",
        KeyCode::NumLock => "",
        KeyCode::PrintScreen => "",
        KeyCode::Pause => "",
        KeyCode::KeypadBegin => "",
        KeyCode::Modifier(_) => "",
    };

    let mut modifiers = Vec::with_capacity(3);

    if key_event.modifiers.intersects(KeyModifiers::CONTROL) {
        modifiers.push("C");
    }

    if key_event.modifiers.intersects(KeyModifiers::SHIFT) {
        modifiers.push("S");
    }

    if key_event.modifiers.intersects(KeyModifiers::ALT) {
        modifiers.push("A");
    }

    let mut key = modifiers.join("-");

    if !key.is_empty() {
        key.push('-');
    }
    if modifiers.is_empty() && key_code.len() > 1 {
        key.push('<');
        key.push_str(key_code);
        key.push('>');
    } else {
        key.push_str(key_code);
    }

    if !modifiers.is_empty() {
        key = format!("<{key}>");
    }

    key
}
