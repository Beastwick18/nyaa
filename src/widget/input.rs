use std::cmp::{max, min};

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
    pub cursor: usize,
    pub max_len: usize,
    pub validator: Option<fn(char) -> bool>,
}

impl InputWidget {
    pub fn new(max_len: usize, validator: Option<fn(char) -> bool>) -> Self {
        InputWidget {
            input: "".to_owned(),
            cursor: 0,
            max_len,
            validator,
        }
    }

    pub fn show_cursor(&self, f: &mut Frame, area: Rect) {
        f.set_cursor(
            min(area.x + self.cursor as u16, area.x + area.width - 1),
            area.y,
        );
    }
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
                        if !validator(*c) {
                            return; // If character is invalid, ignore it
                        }
                    }
                    if self.input.len() < self.max_len {
                        self.input.insert(self.cursor, *c);
                        self.cursor += c.width_cjk().unwrap_or(0);
                    }
                }
                (Char('b') | Left, &KeyModifiers::CONTROL) => {
                    let non_space = self.input[..min(self.cursor, self.input.len())]
                        .rfind(|item| item != ' ')
                        .unwrap_or(0);
                    self.cursor = match self.input[..non_space].rfind(|item| item == ' ') {
                        Some(pos) => pos + 1,
                        None => 0,
                    };
                }
                (Char('w') | Right, &KeyModifiers::CONTROL) => {
                    let idx = min(self.cursor + 1, self.input.len());
                    self.cursor = match self.input[idx..].find(|item| item == ' ') {
                        Some(pos) => self.cursor + pos + 2,
                        None => self.input.len(),
                    };
                }
                (Delete, &KeyModifiers::CONTROL | &KeyModifiers::ALT) => {
                    let idx = min(self.cursor + 1, self.input.len());
                    let new_cursor = match self.input[idx..].find(|item| item == ' ') {
                        Some(pos) => self.cursor + pos + 2,
                        None => self.input.len(),
                    };
                    self.input.replace_range(self.cursor..new_cursor, "");
                }
                (Backspace, &KeyModifiers::CONTROL | &KeyModifiers::ALT) => {
                    let cursor = min(self.cursor, self.input.len());
                    let non_space = self.input[..cursor].rfind(|i| i != ' ').unwrap_or(0);
                    self.cursor = match self.input[..non_space].rfind(|i| i == ' ') {
                        Some(pos) => pos + 1,
                        None => 0,
                    };
                    self.input.replace_range(self.cursor..cursor, "");
                }
                (Backspace, &KeyModifiers::NONE) => {
                    if !self.input.is_empty() && self.cursor > 0 {
                        self.input.remove(self.cursor - 1);
                        self.cursor -= 1;
                    }
                }
                (Delete, &KeyModifiers::NONE) => {
                    if !self.input.is_empty() && self.cursor < self.input.len() {
                        self.input.remove(self.cursor);
                    }
                }
                (Left, &KeyModifiers::NONE)
                | (Char('h'), &KeyModifiers::CONTROL | &KeyModifiers::ALT) => {
                    self.cursor = max(self.cursor, 1) - 1;
                }
                (Right, &KeyModifiers::NONE)
                | (Char('l'), &KeyModifiers::CONTROL | &KeyModifiers::ALT) => {
                    self.cursor = min(self.cursor + 1, self.input.len());
                }
                (End, &KeyModifiers::NONE) | (Char('e'), &KeyModifiers::CONTROL) => {
                    self.cursor = self.input.len();
                }
                (Home, &KeyModifiers::NONE) | (Char('a'), &KeyModifiers::CONTROL) => {
                    self.cursor = 0;
                }
                (Char('u'), &KeyModifiers::CONTROL) => {
                    self.cursor = 0;
                    self.input = "".to_owned();
                }
                _ => {}
            };
        }
        if let Event::Paste(mut p) = evt.to_owned() {
            if let Some(validator) = self.validator {
                // Remove invalid chars
                p = p.chars().filter(|c| validator(*c)).collect();
            }
            self.input = format!(
                "{}{}{}",
                &self.input[..self.cursor],
                p,
                &self.input[self.cursor..]
            );
            if self.input.len() > self.max_len {
                self.input = self.input[..self.max_len].to_owned();
            }
            self.cursor = min(self.cursor + p.len(), self.max_len);
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
