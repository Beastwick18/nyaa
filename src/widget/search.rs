use std::cmp::{max, min};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Clear, Paragraph},
    Frame,
};
use unicode_width::UnicodeWidthChar;

use crate::app::{App, Mode};

use super::create_block;

pub struct SearchWidget {
    pub input: String,
    pub focused: bool,
    pub cursor: usize,
}

impl Default for SearchWidget {
    fn default() -> Self {
        SearchWidget {
            input: "".to_owned(),
            focused: false,
            cursor: 0,
        }
    }
}

impl super::Widget for SearchWidget {
    fn draw(&self, f: &mut Frame, app: &App, area: Rect) {
        let width = self.input.len();
        let fwidth = f.size().width as usize - 2;
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
        let p = Paragraph::new(visible)
            .block(create_block(app.theme, app.mode == Mode::Search).title("Search"));
        f.render_widget(Clear, area);
        f.render_widget(p, area);

        let text = Paragraph::new(Line::from(vec![
            Span::raw("Press "),
            Span::styled("F1", Style::new().bold()),
            Span::raw(" or "),
            Span::styled("?", Style::new().bold()),
            Span::raw(" for help"),
        ]));
        let right = Rect::new(area.right() - 23, area.top(), 23, 1);
        f.render_widget(text, right);
        if app.mode == Mode::Search {
            // Render cursor if in editing mode
            f.set_cursor(
                min(area.x + self.cursor as u16 + 1, area.x + area.width - 2),
                area.y + 1,
            );
        }
    }

    fn handle_event(&mut self, app: &mut crate::app::App, evt: &Event) {
        if let Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            modifiers,
            ..
        }) = evt
        {
            use KeyCode::*;
            match (code, modifiers) {
                (Esc, &KeyModifiers::NONE) => {
                    app.mode = Mode::Normal;
                }
                (Char(c), &KeyModifiers::NONE | &KeyModifiers::SHIFT) => {
                    self.input.insert(self.cursor, *c);
                    self.cursor += c.width_cjk().unwrap_or(0);
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
                    let non_space = self.input[..min(self.cursor, self.input.len())]
                        .rfind(|item| item != ' ')
                        .unwrap_or(0);
                    let prev_cursor = self.cursor;
                    self.cursor = match self.input[..non_space].rfind(|item| item == ' ') {
                        Some(pos) => pos + 1,
                        None => 0,
                    };
                    self.input.replace_range(self.cursor..prev_cursor, "");
                }
                (Backspace, &KeyModifiers::NONE) => {
                    if !self.input.is_empty() && self.cursor > 0 {
                        self.input.remove(self.cursor - 1);
                        self.cursor -= 1;
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
                (Enter, &KeyModifiers::NONE) => {
                    app.mode = Mode::Loading;
                }
                _ => {}
            };
        }
    }

    fn get_help() -> Option<Vec<(&'static str, &'static str)>> {
        Some(vec![
            ("Enter", "Confirm"),
            ("Esc", "Stop"),
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
