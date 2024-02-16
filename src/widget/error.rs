use crossterm::event::{Event, KeyEvent, KeyEventKind};
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::{App, Mode};

use super::Widget;

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
        let center = super::centered_rect(30, 8, area);
        let clear = super::centered_rect(center.width + 2, center.height, area);
        let p = Paragraph::new(self.error.to_owned())
            .block(
                Block::new()
                    .border_style(Style::new().fg(app.theme.border_focused_color))
                    .borders(Borders::ALL)
                    .border_type(app.theme.border)
                    .title("Error"),
            )
            .fg(app.theme.fg)
            .bg(app.theme.bg);
        f.render_widget(Clear, clear);
        f.render_widget(Block::new().bg(app.theme.bg), clear);
        f.render_widget(p, center);
    }

    fn handle_event(&mut self, app: &mut crate::app::App, e: &crossterm::event::Event) {
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
}
