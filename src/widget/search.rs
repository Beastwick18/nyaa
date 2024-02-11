use std::cmp::min;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{layout::Rect, widgets::Paragraph, Frame};
use unicode_width::UnicodeWidthStr;

use crate::{
    app::{App, Mode},
    ui,
};

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
        let visible: String;
        // Try to insert ellipsis if input is too long (visual only)
        if width >= fwidth {
            let idx = width - fwidth + 2;
            if let Some(sub) = self.input.get(idx..) {
                visible = format!("…{}", sub);
            } else {
                visible = self.input.to_owned();
            }
        } else {
            visible = self.input.to_owned();
        }
        let p = Paragraph::new(visible).block(
            match app.mode {
                Mode::Search => ui::HI_BLOCK.clone(),
                _ => ui::DEFAULT_BLOCK.clone(),
            }
            .title("Search"),
        );
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
