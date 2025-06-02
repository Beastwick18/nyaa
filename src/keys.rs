use std::fmt::Display;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use derive_more::{Deref, DerefMut};
use enum_assoc::Assoc;
use indexmap::IndexMap;
use ratatui::style::Color;
use serde::{Deserialize, Deserializer};
use strum::IntoEnumIterator;

use crate::{
    action::UserAction,
    app::{InputMode, Mode},
    color::to_rgb,
};

// Keys that cannot be used in combos, and will cancel the current combo.
// These keys may be used by themselves, as a single key keybind
pub static NON_COMBO: &[KeyCode] = &[KeyCode::Esc];

#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ModeOrDefault {
    #[serde(rename = "_")]
    Default,
    #[serde(untagged)]
    Mode(Mode),
}

/// KeyBindings are a collection of *key*-*user action* pairs, seperated by mode
#[derive(Clone, Debug, Default, Deref, DerefMut)]
// pub struct KeyBindings(pub IndexMap<Mode, IndexMap<Vec<KeyEvent>, OneOrManyActions>>);
pub struct KeyBindings(pub IndexMap<InputMode, IndexMap<Mode, Keymap>>);
#[derive(Clone, Debug, Default, Deref, DerefMut)]
pub struct KeyBindingsWithDefaults(pub IndexMap<InputMode, IndexMap<ModeOrDefault, Keymap>>);

#[derive(Clone, Debug, Default, Deref, DerefMut)]
pub struct Keymap(pub IndexMap<Vec<KeyEvent>, OneOrManyActions>);

impl KeyBindings {
    pub fn keymap(&self, mode: &Mode, input_mode: &InputMode) -> &Keymap {
        self.get(input_mode).and_then(|x| x.get(mode)).unwrap()
    }

    pub fn action(
        &self,
        keycombo: &Vec<KeyEvent>,
        mode: &Mode,
        input_mode: &InputMode,
    ) -> Option<&OneOrManyActions> {
        self.keymap(mode, input_mode).get(keycombo)
    }

    pub fn possible_actions<'a>(
        &'a self,
        keycombo: &'a [KeyEvent],
        mode: &Mode,
        input_mode: &InputMode,
    ) -> impl Iterator<
        Item = (
            &'a std::vec::Vec<crossterm::event::KeyEvent>,
            &'a OneOrManyActions,
        ),
    > {
        self.keymap(mode, input_mode)
            .iter()
            .filter(|(events, _action)| events.starts_with(keycombo))
    }
}

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
    #[assoc(color = to_rgb(Color::White))]
    Pending,
    #[assoc(color = to_rgb(Color::Cyan))]
    Successful,
    #[default]
    #[assoc(color = to_rgb(Color::DarkGray))]
    Cancelled,
    #[assoc(color = to_rgb(Color::DarkGray))]
    Inserted, // handled by insert mode
    #[assoc(color = to_rgb(Color::Red))]
    Unmatched,
}

#[derive(Assoc, Clone, Debug, Deserialize, PartialEq, Eq)]
#[func(pub fn multiplier(&self) -> u8)]
#[func(pub fn actions(&self) -> Vec<UserAction>)]
pub enum OneOrManyActions {
    #[assoc(multiplier = 1, actions = vec![])]
    Nop, // Do nothing

    #[serde(untagged)]
    #[assoc(multiplier = 1, actions = vec![_0.clone()])]
    One(UserAction), // Single action

    #[serde(untagged)]
    #[assoc(multiplier = 1, actions = _0.clone())]
    Many(Vec<UserAction>), // Sequence of actions

    #[serde(untagged)]
    #[assoc(multiplier = *_0, actions = vec![_1.clone()])]
    Repeat(u8, UserAction), // Action with multiplier
}

impl Display for OneOrManyActions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            OneOrManyActions::Nop => OneOrManyActions::Nop.to_string(),
            OneOrManyActions::One(user_action) => user_action.name(),
            OneOrManyActions::Many(user_actions) => user_actions
                .iter()
                .map(UserAction::name)
                .collect::<Vec<String>>()
                .join(", "),
            OneOrManyActions::Repeat(n, user_action) => format!("{n}×{}", user_action.name()),
        };
        write!(f, "{}", s)
    }
}

impl<'de> Deserialize<'de> for KeyBindings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize as KeyBindingsWithDefaults
        let parsed_map = IndexMap::<
            InputMode,
            IndexMap<ModeOrDefault, IndexMap<String, OneOrManyActions>>,
        >::deserialize(deserializer)?;

        // Parse keybinds
        let keybindings = parsed_map
            .into_iter()
            .map(|(mode, mode_map)| {
                let input_mode_mapped = mode_map
                    .into_iter()
                    .map(|(input_mode, inner_map)| {
                        let converted_inner_map = inner_map
                            .into_iter()
                            .map(|(key_str, cmd)| (parse_key_sequence(&key_str).unwrap(), cmd))
                            .collect();
                        (input_mode, Keymap(converted_inner_map))
                    })
                    .collect();
                (mode, input_mode_mapped)
            })
            .collect();
        let keybindings = KeyBindingsWithDefaults(keybindings);

        // Go through each input mode, and extend default keybinds into each Mode's keybinds
        let mut combined_keybindings = KeyBindings::default();
        for (input_mode, mode_bindings) in keybindings.iter() {
            let general_action = mode_bindings
                .get(&ModeOrDefault::Default)
                .cloned()
                .unwrap_or_default();
            let mut new_mode_bindings = IndexMap::new();

            for (mode, keymap) in mode_bindings.iter() {
                if let ModeOrDefault::Mode(mode) = mode {
                    let mut cloned_general_action = general_action.clone();
                    cloned_general_action.extend(keymap.0.clone());
                    new_mode_bindings.insert(*mode, cloned_general_action);
                }
            }

            // Use general keybinds for missing modes
            for mode in Mode::iter() {
                if !new_mode_bindings.contains_key(&mode) {
                    let cloned_general_action = general_action.clone();
                    new_mode_bindings.insert(mode, cloned_general_action);
                }
            }

            combined_keybindings.insert(*input_mode, new_mode_bindings);
        }

        Ok(combined_keybindings)
    }
}

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
        "enter" | "cr" => KeyCode::Enter,
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
        "bs" => KeyCode::Backspace,
        "delete" => KeyCode::Delete,
        "del" => KeyCode::Delete,
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
        "hyphen" | "minus" | "-" => KeyCode::Char('-'),
        "lt" => KeyCode::Char('<'),
        "gt" => KeyCode::Char('>'),
        "tab" if modifiers.contains(KeyModifiers::SHIFT) => KeyCode::BackTab,
        "tab" => KeyCode::Tab,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_keyevent() {
        assert!(parse_key_event("").is_err());
    }

    #[test]
    fn test_empty_key_sequence() {
        assert_eq!(parse_key_sequence("").unwrap(), vec![]);
    }

    #[test]
    fn test_single_key_sequence() {
        assert_eq!(
            parse_key_sequence("x").unwrap(),
            vec![KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE)]
        );
    }

    #[test]
    fn test_single_key_sequence_modifier() {
        assert_eq!(
            parse_key_sequence("<Ctrl-x>").unwrap(),
            vec![KeyEvent::new(KeyCode::Char('x'), KeyModifiers::CONTROL)]
        );
    }

    #[test]
    fn test_multiple_key_sequence_modifier() {
        assert!(parse_key_sequence("<Ctrl-xyz>").is_err());
    }

    #[test]
    fn test_complex_key_sequence() {
        assert_eq!(
            parse_key_sequence("<Ctrl-x>abc<Shift-y>").unwrap(),
            vec![
                KeyEvent::new(KeyCode::Char('x'), KeyModifiers::CONTROL),
                KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Char('Y'), KeyModifiers::SHIFT)
            ]
        );
    }

    #[test]
    fn test_parse_toml() {
        let raw_toml = r#"
            [Insert._]
            "a" = { "NotifyInfo" = "Message 1" }
            "b" = { "NotifyInfo" = "Message 1" }

            [Normal._]
            "c" = { "NotifyInfo" = "Message 2" }
            "d" = { "NotifyInfo" = "Message 2" }

            [Insert.Search]
            "b" = { "NotifyInfo" = "Message 1 override" }
            "e" = { "NotifyInfo" = "Message 1 unique" }

            [Normal.Home]
            "d" = { "NotifyInfo" = "Message 2 override" }
            "f" = { "NotifyInfo" = "Message 2 unique" }
        "#;

        let keys: KeyBindings = toml::from_str(raw_toml).unwrap();

        let get_action = |im, m, ch| {
            keys.get(&im)
                .unwrap()
                .get(&m)
                .unwrap()
                .get(&vec![KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE)])
                .unwrap()
        };

        assert_eq!(
            get_action(InputMode::Insert, Mode::Search, 'a'),
            &OneOrManyActions::One(UserAction::NotifyInfo("Message 1".to_string()))
        );

        assert_eq!(
            get_action(InputMode::Insert, Mode::Search, 'b'),
            &OneOrManyActions::One(UserAction::NotifyInfo("Message 1 override".to_string()))
        );

        assert_eq!(
            get_action(InputMode::Normal, Mode::Home, 'c'),
            &OneOrManyActions::One(UserAction::NotifyInfo("Message 2".to_string()))
        );

        assert_eq!(
            get_action(InputMode::Normal, Mode::Home, 'd'),
            &OneOrManyActions::One(UserAction::NotifyInfo("Message 2 override".to_string()))
        );

        assert_eq!(
            get_action(InputMode::Insert, Mode::Search, 'e'),
            &OneOrManyActions::One(UserAction::NotifyInfo("Message 1 unique".to_string()))
        );

        assert_eq!(
            get_action(InputMode::Normal, Mode::Home, 'f'),
            &OneOrManyActions::One(UserAction::NotifyInfo("Message 2 unique".to_string()))
        );
    }
}
