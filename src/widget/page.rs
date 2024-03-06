use std::cmp::{max, min};

use crate::app::{App, LoadType, Mode};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Margin, Rect},
    widgets::{Paragraph, Widget as _},
    Frame,
};

use super::{
    border_block,
    input::{self, InputWidget},
    Widget,
};

pub struct PagePopup {
    pub input: InputWidget,
}

impl Default for PagePopup {
    fn default() -> Self {
        PagePopup {
            input: InputWidget::new(3, Some(|e| e.is_numeric())),
        }
    }
}

impl Widget for PagePopup {
    fn draw(&self, f: &mut Frame, app: &App, area: Rect) {
        let buf = f.buffer_mut();
        let center = super::centered_rect(13, 3, area);
        let clear = super::centered_rect(center.width + 2, center.height, area);
        let page_p = Paragraph::new(self.input.input.clone());
        let indicator = Paragraph::new(">").block(border_block(app.theme, true).title("Goto Page"));
        super::clear(clear, buf, app.theme.bg);
        indicator.render(center, buf);

        let input_area = center.inner(&Margin {
            vertical: 1,
            horizontal: 1,
        });
        let input_area = Rect::new(
            input_area.x + 2,
            input_area.y,
            input_area.width,
            input_area.height,
        );
        page_p.render(input_area, buf);

        if app.mode == Mode::Page {
            self.input.show_cursor(f, input_area);
        }
    }

    fn handle_event(&mut self, app: &mut crate::app::App, e: &crossterm::event::Event) {
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
                KeyCode::Enter => {
                    app.page = max(min(self.input.input.parse().unwrap_or(1), app.last_page), 1);
                    app.mode = Mode::Loading(LoadType::Searching);

                    // Clear input on enter
                    self.input.input = "".to_owned();
                    self.input.cursor = 0;
                }
                _ => {
                    self.input.handle_event(app, e);
                }
            }
        }
    }

    fn get_help() -> Option<Vec<(&'static str, &'static str)>> {
        let mut search_help = vec![("Enter", "Confirm"), ("Esc", "Stop")];
        if let Some(input_help) = input::InputWidget::get_help() {
            search_help.extend(input_help);
        }
        Some(search_help)
    }
}
