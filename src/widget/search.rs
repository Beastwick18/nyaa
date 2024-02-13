use std::cmp::min;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use unicode_width::UnicodeWidthStr;

use crate::app::{App, Mode};

pub struct SearchWidget {
    pub input: String,
    pub focused: bool,
}

impl Default for SearchWidget {
    fn default() -> Self {
        SearchWidget {
            input: "".to_owned(),
            focused: false,
        }
    }
}

impl super::Widget for SearchWidget {
    fn draw(&self, f: &mut Frame, app: &App, area: Rect) {
        let width = self.input.width();
        let fwidth = f.size().width as usize - 2;
        // let visible: String;
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
        let p = Paragraph::new(visible).block(
            match app.mode {
                Mode::Search => Block::new()
                    .borders(Borders::ALL)
                    .border_type(app.theme.border)
                    .border_style(Style::new().fg(app.theme.border_focused_color)),
                _ => Block::new()
                    .borders(Borders::ALL)
                    .border_type(app.theme.border)
                    .border_style(Style::new().fg(app.theme.border_color)),
            }
            .fg(app.theme.fg)
            .bg(app.theme.bg)
            .title("Search"),
        );
        f.render_widget(Clear, area);
        f.render_widget(p, area);
        match app.mode {
            Mode::Search => {
                // Render cursor if in editing mode
                f.set_cursor(
                    min(
                        area.x + self.input.width() as u16 + 1,
                        area.x + area.width - 2,
                    ),
                    area.y + 1,
                );
            }
            _ => {}
        }
    }

    fn handle_event(&mut self, app: &mut crate::app::App, e: &Event) {
        if let Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            ..
        }) = e
        {
            match code {
                KeyCode::Esc => {
                    app.mode = Mode::Normal;
                }
                KeyCode::Char(c) => {
                    self.input.push(*c);
                }
                KeyCode::Backspace => {
                    self.input.pop();
                }
                _ => {}
            };
        }
    }
}
