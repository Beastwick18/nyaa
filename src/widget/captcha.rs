use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Direction, Layout, Margin, Rect},
    widgets::StatefulWidget as _,
    Frame,
};
use ratatui_image::{protocol::StatefulProtocol, StatefulImage};

use crate::app::{Context, LoadType, Mode};

use super::{input::InputWidget, Widget};

pub struct CaptchaPopup {
    pub image: Option<Box<dyn StatefulProtocol>>,
    pub input: InputWidget,
}

impl Default for CaptchaPopup {
    fn default() -> Self {
        Self {
            image: None,
            input: InputWidget::new(32, None),
        }
    }
}

impl InputWidget {
    pub fn clear(&mut self) {
        self.input.clear();
        self.cursor = 0;
    }
}

impl Widget for CaptchaPopup {
    fn draw(&mut self, f: &mut Frame, ctx: &Context, area: Rect) {
        let center = area.inner(&Margin {
            horizontal: 4,
            vertical: 4,
        });
        super::clear(center, f.buffer_mut(), ctx.theme.bg);
        let layout = Layout::new(
            Direction::Vertical,
            [Constraint::Fill(1), Constraint::Length(3)],
        )
        .split(center);
        if let Some(img) = self.image.as_mut() {
            f.render_widget(
                super::border_block(&ctx.theme, true).title("Captcha"),
                layout[0],
            );
            StatefulImage::new(None).render(
                layout[0].inner(&Margin {
                    horizontal: 1,
                    vertical: 1,
                }),
                f.buffer_mut(),
                img,
            );
        }
        f.render_widget(
            super::border_block(&ctx.theme, true).title("Enter Captcha solution"),
            layout[1],
        );

        let input_area = layout[1].inner(&Margin {
            horizontal: 1,
            vertical: 1,
        });
        self.input.draw(f, ctx, input_area);
        self.input.show_cursor(f, input_area);
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
                    ctx.mode = Mode::Loading(LoadType::SolvingCaptcha(self.input.input.clone()));
                }
                _ => {}
            }
        }
        self.input.handle_event(ctx, e);
    }

    fn get_help() -> Option<Vec<(&'static str, &'static str)>> {
        Some(vec![
            ("Enter", "Confirm"),
            ("Esc, f, q", "Close"),
            ("g", "Top"),
            ("G", "Bottom"),
            ("j, ↓", "Down"),
            ("k, ↑", "Up"),
        ])
    }
}
