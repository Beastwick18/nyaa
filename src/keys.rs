use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use derive_deref::{Deref, DerefMut};
use indexmap::IndexMap;
use serde::{Deserialize, Deserializer};

use crate::{action::UserAction, app::Mode};

/// KeyBindings are a collection of *key*-*user action* pairs
#[derive(Clone, Debug, Default, Deref, DerefMut)]
pub struct KeyBindings(pub IndexMap<Mode, IndexMap<Vec<KeyEvent>, UserAction>>);

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum UserActionWrapped {
    Unit(UserAction),
    // Other(String), // TODO: Parse custom syntax, like "Action(arg1, ...)"
}

impl<'de> Deserialize<'de> for KeyBindings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let parsed_map =
            IndexMap::<Mode, IndexMap<String, UserActionWrapped>>::deserialize(deserializer)?;

        let keybindings = parsed_map
            .into_iter()
            .map(|(mode, inner_map)| {
                let converted_inner_map = inner_map
                    .into_iter()
                    .map(|(key_str, cmd)| (key_str, parse_wrapped_actions(cmd).unwrap()))
                    .map(|(key_str, cmd)| (parse_key_sequence(&key_str).unwrap(), cmd))
                    .collect();
                (mode, converted_inner_map)
            })
            .collect();

        Ok(KeyBindings(keybindings))
    }
}

fn parse_wrapped_actions(cmd: UserActionWrapped) -> Result<UserAction, String> {
    Ok(match cmd {
        UserActionWrapped::Unit(unit) => unit,
        // UserActionWrapped::Other(other) => parse_other_action_simple(other)?,
    })
}

// TODO: For now, only handle simple case of Enum1(Enum2) or Enum1(String)
fn parse_other_action_simple(other: String) -> Result<UserAction, String> {
    let tokens = other.split_ascii_whitespace().collect::<Vec<&str>>();
    match tokens.as_slice() {
        [key, val] => {
            let mut table = toml::Table::new();
            table.insert(key.to_string(), toml::Value::String(val.to_string()));
            Ok(table.try_into().unwrap())
        }
        _ => Err("UserAction is not properly formatted".to_string()),
    }
}

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
    if raw.chars().filter(|c| *c == '>').count() != raw.chars().filter(|c| *c == '<').count() {
        return Err(format!("Unable to parse `{}`", raw));
    }
    let raw = if !raw.contains("><") {
        let raw = raw.strip_prefix('<').unwrap_or(raw);
        let raw = raw.strip_prefix('>').unwrap_or(raw);
        raw
    } else {
        raw
    };
    let sequences = raw
        .split("><")
        .map(|seq| {
            if let Some(s) = seq.strip_prefix('<') {
                s
            } else if let Some(s) = seq.strip_suffix('>') {
                s
            } else {
                seq
            }
        })
        .collect::<Vec<_>>();

    sequences.into_iter().map(parse_key_event).collect()
}

// pub fn key_event_to_string(key_event: &KeyEvent) -> String {
//     let char;
//     let key_code = match key_event.code {
//         KeyCode::Backspace => "backspace",
//         KeyCode::Enter => "enter",
//         KeyCode::Left => "left",
//         KeyCode::Right => "right",
//         KeyCode::Up => "up",
//         KeyCode::Down => "down",
//         KeyCode::Home => "home",
//         KeyCode::End => "end",
//         KeyCode::PageUp => "pageup",
//         KeyCode::PageDown => "pagedown",
//         KeyCode::Tab => "tab",
//         KeyCode::BackTab => "backtab",
//         KeyCode::Delete => "delete",
//         KeyCode::Insert => "insert",
//         KeyCode::F(c) => {
//             char = format!("f({c})");
//             &char
//         }
//         KeyCode::Char(' ') => "space",
//         KeyCode::Char(c) => {
//             char = c.to_string();
//             &char
//         }
//         KeyCode::Esc => "esc",
//         KeyCode::Null => "",
//         KeyCode::CapsLock => "",
//         KeyCode::Menu => "",
//         KeyCode::ScrollLock => "",
//         KeyCode::Media(_) => "",
//         KeyCode::NumLock => "",
//         KeyCode::PrintScreen => "",
//         KeyCode::Pause => "",
//         KeyCode::KeypadBegin => "",
//         KeyCode::Modifier(_) => "",
//     };
//
//     let mut modifiers = Vec::with_capacity(3);
//
//     if key_event.modifiers.intersects(KeyModifiers::CONTROL) {
//         modifiers.push("ctrl");
//     }
//
//     if key_event.modifiers.intersects(KeyModifiers::SHIFT) {
//         modifiers.push("shift");
//     }
//
//     if key_event.modifiers.intersects(KeyModifiers::ALT) {
//         modifiers.push("alt");
//     }
//
//     let mut key = modifiers.join("-");
//
//     if !key.is_empty() {
//         key.push('-');
//     }
//     key.push_str(key_code);
//
//     key
// }
