use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Margin, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Clear, Paragraph},
    Frame,
};

use crate::app::{App, LoadType, Mode};

use super::{
    create_block,
    input::{self, InputWidget},
};

pub struct SearchWidget {
    pub input: InputWidget,
}

impl Default for SearchWidget {
    fn default() -> Self {
        SearchWidget {
            input: InputWidget::new(300, Some(|_| true)),
        }
    }
}

impl super::Widget for SearchWidget {
    fn draw(&self, f: &mut Frame, app: &App, area: Rect) {
        // let fwidth = f.size().width as usize - 2;
        // Try to insert ellipsis if input is too long (visual only)
        let block = create_block(app.theme, app.mode == Mode::Search).title("Search");
        f.render_widget(Clear, area);
        f.render_widget(block, area);
        let input_area = area.inner(&Margin {
            vertical: 1,
            horizontal: 1,
        });
        self.input.draw(f, app, input_area);
        if app.mode == Mode::Search {
            self.input.show_cursor(f, input_area);
        }

        let text = Paragraph::new(Line::from(vec![
            Span::raw("Press "),
            Span::styled("F1", Style::new().bold()),
            Span::raw(" or "),
            Span::styled("?", Style::new().bold()),
            Span::raw(" for help"),
        ]));
        let right = Rect::new(area.right() - 23, area.top(), 23, 1);
        f.render_widget(text, right);
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
                (Enter, &KeyModifiers::NONE) => {
                    app.mode = Mode::Loading(LoadType::Searching);
                    app.page = 1; // Go back to first page
                }
                _ => {
                    self.input.handle_event(app, evt);
                }
            };
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
