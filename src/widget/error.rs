use std::cmp::max;

use crossterm::event::{Event, KeyEvent, KeyEventKind};
use ratatui::{layout::Rect, style::Stylize, widgets::Paragraph, Frame};

use crate::app::{App, Mode};

use super::{create_block, Widget};

pub struct ErrorPopup {
    pub error: String,
}

impl ErrorPopup {
    pub fn with_error(&mut self, error: String) {
        self.error = error;
    }
}

impl Default for ErrorPopup {
    fn default() -> Self {
        ErrorPopup {
            error: "".to_owned(),
        }
    }
}

impl Widget for ErrorPopup {
    fn draw(&self, f: &mut Frame, app: &App, area: Rect) {
        let lines = self.error.split("\n");
        let max_line = lines.clone().fold(30, |acc, e| max(e.len(), acc)) as u16 + 2;
        let height = lines.count() as u16 + 2;
        let center = super::centered_rect(max_line, height, area);
        let clear = super::centered_rect(center.width + 2, center.height, area);
        let p = Paragraph::new(self.error.to_owned()).block(
            create_block(app.theme, true)
                .fg(app.theme.remake)
                .title("Error"),
        );
        super::clear(f, clear, app.theme.bg);
        f.render_widget(p, center);
    }

    fn handle_event(&mut self, app: &mut App, e: &Event) {
        if let Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            ..
        }) = e
        {
            match code {
                _ => {
                    if app.errors.len() == 0 {
                        app.mode = Mode::Normal;
                    }
                }
            }
        }
    }

    fn get_help() -> Option<Vec<(&'static str, &'static str)>> {
        None
    }
}
