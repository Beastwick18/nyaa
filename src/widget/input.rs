use std::{
    cmp::{max, min},
    ops::RangeBounds,
};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::Rect,
    widgets::{Paragraph, Widget},
    Frame,
};
use unicode_width::UnicodeWidthChar;

use crate::app::Context;

pub struct InputWidget {
    pub input: String,
    pub char_idx: usize,
    pub cursor: usize,
    pub max_len: usize,
    pub validator: Option<fn(&char) -> bool>,
}

impl InputWidget {
    pub fn new(max_len: usize, validator: Option<fn(&char) -> bool>) -> Self {
        InputWidget {
            input: "".to_owned(),
            char_idx: 0,
            cursor: 0,
            max_len,
            validator,
        }
    }

    pub fn show_cursor(&self, f: &mut Frame, area: Rect) {
        f.set_cursor(
            min(area.x + self.cursor as u16, area.x + area.width.max(1) - 1),
            area.y,
        );
    }

    pub fn set_cursor(&mut self, idx: usize) {
        self.char_idx = idx.min(self.max_len);
        self.cursor = pos_of_nth_char(&self.input, self.char_idx);
    }

    pub fn clear(&mut self) {
        self.input.clear();
        self.cursor = 0;
        self.char_idx = 0;
    }
}

fn pos_of_nth_char(s: &String, idx: usize) -> usize {
    s.chars()
        .take(idx)
        .fold(0, |acc, c| acc + c.width().unwrap_or(0))
}

fn without_nth_char(s: &String, idx: usize) -> String {
    s.chars()
        .enumerate()
        .filter_map(|(i, c)| if i != idx { Some(c) } else { None })
        .collect::<String>()
}

fn without_range(s: &String, range: impl RangeBounds<usize>) -> String {
    let mut vec = s.chars().collect::<Vec<char>>();
    vec.drain(range);
    vec.into_iter().collect()
}

fn insert_char(s: &String, idx: usize, x: char) -> String {
    let mut vec = s.chars().collect::<Vec<char>>();
    vec.insert(idx, x);
    vec.into_iter().collect()
}

impl super::Widget for InputWidget {
    fn draw(&mut self, f: &mut Frame, _ctx: &Context, area: Rect) {
        let width = self.input.len();
        let fwidth = area.width as usize;
        // Try to insert ellipsis if input is too long (visual only)
        let visible = if width >= fwidth {
            let idx = width - fwidth + 2;
            match self.input.get(idx..) {
                Some(sub) => format!("…{}", sub),
                None => self.input.to_owned(),
            }
        } else {
            self.input.to_owned()
        };
        let p = Paragraph::new(visible);
        p.render(area, f.buffer_mut());
    }

    fn handle_event(&mut self, _ctx: &mut Context, evt: &Event) {
        if let Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            modifiers,
            ..
        }) = evt
        {
            use KeyCode::*;
            match (code, modifiers) {
                (Char(c), &KeyModifiers::NONE | &KeyModifiers::SHIFT) => {
                    if let Some(validator) = &self.validator {
                        if !validator(c) {
                            return; // If character is invalid, ignore it
                        }
                    }
                    if self.input.chars().count() < self.max_len {
                        self.input = insert_char(&self.input, self.char_idx, *c);
                        self.char_idx += 1;
                    }
                }
                (Char('b') | Left, &KeyModifiers::CONTROL) => {
                    let cursor = min(self.char_idx, self.input.chars().count());
                    // Find the first non-space character before the cursor
                    let non_space = self
                        .input
                        .chars()
                        .take(cursor)
                        .collect::<Vec<char>>()
                        .into_iter()
                        .rposition(|c| c != ' ')
                        .unwrap_or(0);

                    // Find the first space character before the first non-space character
                    self.char_idx = self
                        .input
                        .chars()
                        .take(non_space)
                        .collect::<Vec<char>>()
                        .into_iter()
                        .rposition(|c| c == ' ')
                        .map(|u| u + 1)
                        .unwrap_or(0);
                }
                (Char('w') | Right, &KeyModifiers::CONTROL) => {
                    let idx = min(self.char_idx + 1, self.input.chars().count());

                    self.char_idx = self
                        .input
                        .chars()
                        .skip(idx)
                        .collect::<Vec<char>>()
                        .into_iter()
                        .position(|c| c == ' ')
                        .map(|u| self.char_idx + u + 2)
                        .unwrap_or(self.input.chars().count());
                }
                (Delete, &KeyModifiers::CONTROL | &KeyModifiers::ALT) => {
                    let idx = min(self.char_idx + 1, self.input.chars().count());

                    let new_cursor = self
                        .input
                        .chars()
                        .skip(idx)
                        .collect::<Vec<char>>()
                        .into_iter()
                        .position(|c| c == ' ')
                        .map(|u| self.char_idx + u + 2)
                        .unwrap_or(self.input.chars().count());
                    self.input = without_range(&self.input, self.char_idx..new_cursor)
                }
                (Backspace, &KeyModifiers::CONTROL | &KeyModifiers::ALT) => {
                    let cursor = min(self.char_idx, self.input.chars().count());
                    // Find the first non-space character before the cursor
                    let non_space = self
                        .input
                        .chars()
                        .take(cursor)
                        .collect::<Vec<char>>()
                        .into_iter()
                        .rposition(|c| c != ' ')
                        .unwrap_or(0);

                    // Find the first space character before the first non-space character
                    self.char_idx = self
                        .input
                        .chars()
                        .take(non_space)
                        .collect::<Vec<char>>()
                        .into_iter()
                        .rposition(|c| c == ' ')
                        .map(|u| u + 1)
                        .unwrap_or(0);
                    self.input = without_range(&self.input, self.char_idx..cursor)
                }
                (Backspace, &KeyModifiers::NONE) => {
                    if !self.input.is_empty() && self.char_idx > 0 {
                        self.char_idx -= 1;
                        self.input = without_nth_char(&self.input, self.char_idx);
                    }
                }
                (Delete, &KeyModifiers::NONE) => {
                    if !self.input.is_empty() && self.char_idx < self.input.chars().count() {
                        self.input = without_nth_char(&self.input, self.char_idx);
                    }
                }
                (Left, &KeyModifiers::NONE)
                | (Char('h'), &KeyModifiers::CONTROL | &KeyModifiers::ALT) => {
                    self.char_idx = max(self.char_idx, 1) - 1;
                }
                (Right, &KeyModifiers::NONE)
                | (Char('l'), &KeyModifiers::CONTROL | &KeyModifiers::ALT) => {
                    self.char_idx = min(self.char_idx + 1, self.input.chars().count());
                }
                (End, &KeyModifiers::NONE) | (Char('e'), &KeyModifiers::CONTROL) => {
                    self.char_idx = self.input.chars().count();
                }
                (Home, &KeyModifiers::NONE) | (Char('a'), &KeyModifiers::CONTROL) => {
                    self.char_idx = 0;
                }
                (Char('u'), &KeyModifiers::CONTROL) => {
                    self.char_idx = 0;
                    "".clone_into(&mut self.input);
                }
                _ => {}
            };
            self.cursor = pos_of_nth_char(&self.input, self.char_idx);
        }
        if let Event::Paste(p) = evt.to_owned() {
            let space_left = self.max_len - self.input.chars().count();
            let p = match self.validator {
                // Remove invalid chars
                Some(v) => p.chars().filter(v).collect(),
                None => p,
            };
            let p: String = p.chars().take(space_left).collect();
            let before: String = self.input.chars().take(self.char_idx).collect();
            let after: String = self.input.chars().skip(self.char_idx).collect();
            self.input = format!("{before}{p}{after}");
            self.char_idx = min(self.char_idx + p.chars().count(), self.max_len);

            self.cursor = pos_of_nth_char(&self.input, self.char_idx);
        }
    }

    fn get_help() -> Option<Vec<(&'static str, &'static str)>> {
        Some(vec![
            ("←, Ctrl-h", "Move left"),
            ("→, Ctrl-l", "Move right"),
            ("Ctrl-u", "Clear search"),
            ("End, Ctrl-e", "End of line"),
            ("Home, Ctrl-a", "Beginning of line"),
            ("Ctrl-b, Ctrl-←", "Back word"),
            ("Ctrl-w, Ctrl-→", "Forward word"),
            ("Ctrl/Alt-Del", "Delete word forward"),
            ("Ctrl/Alt-Backspace", "Delete word backwards"),
            ("Del", "Delete letter forwards"),
            ("Backspace", "Delete letter backwards"),
        ])
    }
}
