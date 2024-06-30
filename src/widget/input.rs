use std::cmp::{max, min};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::Rect,
    style::Stylize,
    text::Line,
    widgets::{Paragraph, Widget},
    Frame,
};

use crate::{app::Context, util::strings};

pub struct InputWidget {
    pub input: String,
    pub char_idx: usize,
    pub char_offset: usize,
    pub cursor: usize,
    pub max_len: usize,
    pub validator: Option<fn(&char) -> bool>,
}

impl InputWidget {
    pub fn new(max_len: usize, validator: Option<fn(&char) -> bool>) -> Self {
        InputWidget {
            input: "".to_owned(),
            char_idx: 0,
            char_offset: 0,
            cursor: 0,
            max_len,
            validator,
        }
    }

    pub fn show_cursor(&self, f: &mut Frame, area: Rect) {
        let cursor = self.get_cursor_pos();

        f.set_cursor(min(area.x + cursor as u16, area.x + area.width), area.y);
    }

    pub fn set_cursor(&mut self, idx: usize) {
        self.char_idx = idx.min(self.max_len);
        self.cursor = strings::pos_of_nth_char(&self.input, self.char_idx);
    }

    pub fn clear(&mut self) {
        self.input.clear();
        self.cursor = 0;
        self.char_idx = 0;
    }

    fn get_cursor_pos(&self) -> usize {
        self.cursor - self.char_offset
    }
}

impl super::Widget for InputWidget {
    fn draw(&mut self, f: &mut Frame, ctx: &Context, area: Rect) {
        let fwidth = area.width as usize;
        // Try to insert ellipsis if input is too long (visual only)
        let (ellipsis, visible, ellipsis_back) = strings::truncate_ellipsis(
            self.input.clone(),
            fwidth,
            ctx.config
                .cursor_padding
                .min((fwidth / 2).saturating_sub(1)),
            self.cursor,
            &mut self.char_offset,
        );
        Paragraph::new(Line::from(vec![
            ellipsis.unwrap_or_default().fg(ctx.theme.border_color),
            visible.into(),
            ellipsis_back.unwrap_or_default().fg(ctx.theme.border_color),
        ]))
        .render(area, f.buffer_mut());
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
                        self.input = strings::insert_char(&self.input, self.char_idx, *c);
                        self.char_idx += 1;
                    }
                }
                (Char('b') | Left, &KeyModifiers::CONTROL) => {
                    self.char_idx = strings::back_word(&self.input, self.char_idx);
                }
                (Char('w') | Right, &KeyModifiers::CONTROL) => {
                    self.char_idx = strings::forward_word(&self.input, self.char_idx);
                }
                (Delete, &KeyModifiers::CONTROL | &KeyModifiers::ALT) => {
                    let new_cursor = strings::forward_word(&self.input, self.char_idx);
                    self.input = strings::without_range(&self.input, self.char_idx..new_cursor)
                }
                (Backspace, &KeyModifiers::CONTROL | &KeyModifiers::ALT) => {
                    let new_cursor = strings::back_word(&self.input, self.char_idx);
                    self.input = strings::without_range(&self.input, new_cursor..self.char_idx);
                    self.char_idx = new_cursor;
                }
                (Backspace, &KeyModifiers::NONE) => {
                    if !self.input.is_empty() && self.char_idx > 0 {
                        self.char_idx -= 1;
                        self.input = strings::without_nth_char(&self.input, self.char_idx);
                    }
                }
                (Delete, &KeyModifiers::NONE) => {
                    if !self.input.is_empty() && self.char_idx < self.input.chars().count() {
                        self.input = strings::without_nth_char(&self.input, self.char_idx);
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
            self.cursor = strings::pos_of_nth_char(&self.input, self.char_idx);
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

            self.cursor = strings::pos_of_nth_char(&self.input, self.char_idx);
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
