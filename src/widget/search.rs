use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Margin, Rect},
    style::Stylize,
    widgets::{Clear, Widget},
    Frame,
};

use crate::{
    app::{Context, LoadType, Mode},
    title,
};

use super::{
    border_block,
    input::{self, InputWidget},
    TitlePosition,
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
    fn draw(&mut self, f: &mut Frame, ctx: &Context, area: Rect) {
        let buf = f.buffer_mut();
        let block = border_block(&ctx.theme, ctx.mode == Mode::Search).title(title!("Search"));
        Clear.render(area, buf);
        block.render(area, buf);
        let input_area = area.inner(&Margin {
            vertical: 1,
            horizontal: 1,
        });

        let help_title = title!(
            "Press ".into();
            "F1".bold();
            " or ".into();
            "?".bold();
            " for help".into();
        );
        if let Some((tr, area)) = TitlePosition::TopRight.try_widget(help_title, area, true) {
            tr.render(area, buf);
        }

        self.input.draw(f, ctx, input_area);
        if ctx.mode == Mode::Search {
            self.input.show_cursor(f, input_area);
        }
    }

    fn handle_event(&mut self, ctx: &mut Context, evt: &Event) {
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
                    ctx.mode = Mode::Normal;
                }
                (Enter, &KeyModifiers::NONE) => {
                    ctx.mode = Mode::Loading(LoadType::Searching);
                    ctx.page = 1; // Go back to first page
                }
                _ => {}
            };
        }
        self.input.handle_event(ctx, evt);
    }

    fn get_help() -> Option<Vec<(&'static str, &'static str)>> {
        let mut search_help = vec![("Enter", "Confirm"), ("Esc", "Stop")];
        if let Some(input_help) = input::InputWidget::get_help() {
            search_help.extend(input_help);
        }
        Some(search_help)
    }
}
