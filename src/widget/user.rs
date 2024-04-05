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

pub struct UserPopup {
    pub input: InputWidget,
}

impl Default for UserPopup {
    fn default() -> Self {
        UserPopup {
            input: InputWidget::new(26, Some(|e| e.is_ascii())),
        }
    }
}

impl Widget for UserPopup {
    fn draw(&mut self, f: &mut Frame, ctx: &Context, area: Rect) {
        let buf = f.buffer_mut();
        let center = super::centered_rect(30, 3, area);
        let clear = super::centered_rect(center.width + 2, center.height, area);
        let page_p = Paragraph::new(self.input.input.clone());
        let indicator = Paragraph::new(">")
            .block(border_block(&ctx.theme, true).title(title!("Posts by User")));
        super::clear(clear, buf, ctx.theme.bg);
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

        if ctx.mode == Mode::User {
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
                }
                KeyCode::Enter => {
                    ctx.user = Some(self.input.input.to_owned());
                    ctx.mode = Mode::Loading(LoadType::Searching);
                }
                _ => {
                    self.input.handle_event(ctx, e);
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
