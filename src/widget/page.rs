use std::cmp::{max, min};

use crate::{
    app::{Context, LoadType, Mode},
    title,
};
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
            input: InputWidget::new(3, Some(char::is_ascii_digit)),
        }
    }
}

impl Widget for PagePopup {
    fn draw(&mut self, f: &mut Frame, ctx: &Context, area: Rect) {
        let buf = f.buffer_mut();
        let center = super::centered_rect(13, 3, area);
        let page_p = Paragraph::new(self.input.input.clone());
        let indicator =
            Paragraph::new(">").block(border_block(&ctx.theme, true).title(title!("Goto Page")));
        super::clear(center, buf, ctx.theme.bg);
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

        if ctx.mode == Mode::Page {
            self.input.show_cursor(f, input_area);
        }
    }

    fn handle_event(&mut self, ctx: &mut Context, e: &Event) {
        if let Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            ..
        }) = e
        {
            match code {
                KeyCode::Esc => {
                    ctx.mode = Mode::Normal;
                    // Clear input on Esc
                    self.input.clear();
                }
                KeyCode::Enter => {
                    ctx.page = max(
                        min(
                            self.input.input.parse().unwrap_or(1),
                            ctx.results.response.last_page,
                        ),
                        1,
                    );
                    ctx.mode = Mode::Loading(LoadType::Searching);

                    // Clear input on Enter
                    self.input.clear();
                }
                _ => {}
            }
        }
        self.input.handle_event(ctx, e);
    }

    fn get_help() -> Option<Vec<(&'static str, &'static str)>> {
        let mut search_help = vec![("Enter", "Confirm"), ("Esc", "Stop")];
        if let Some(input_help) = input::InputWidget::get_help() {
            search_help.extend(input_help);
        }
        Some(search_help)
    }
}
