use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Margin, Rect},
    style::{Color, Stylize},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::{
    action::AppAction,
    app::{Context, Mode},
};

use super::Component;

pub struct ResultsComponent {
    content: String,
}

impl ResultsComponent {
    pub fn new() -> Self {
        Self {
            content: "Hello, results".to_string(),
        }
    }
}

impl Component for ResultsComponent {
    fn update(
        &mut self,
        _ctx: &Context,
        action: &AppAction,
    ) -> color_eyre::eyre::Result<Option<AppAction>> {
        if action == &AppAction::Resume {
            self.content = "Welcome back, user".to_string();
        }
        Ok(None)
    }

    fn on_key(&mut self, ctx: &Context, key: &KeyEvent) -> Result<()> {
        if ctx.mode != Mode::Test {
            return Ok(());
        }

        if let KeyEvent {
            code: KeyCode::Char(c),
            ..
        } = key
        {
            self.content.push(*c);
        }
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let block = Block::new().borders(Borders::ALL);
        let inner = area.inner(Margin::new(1, 1));
        let p = Paragraph::new(" Ctrl-k ")
            .bg(Color::Rgb(24, 24, 24))
            .fg(Color::Rgb(255, 230, 230));
        frame.render_widget(block, area);
        frame.render_widget(&p, Rect::new(inner.x, inner.y, " Ctrl-k ".len() as u16, 1));
        frame.render_widget(
            &p,
            Rect::new(inner.x, inner.y + 2, " Ctrl-k ".len() as u16, 1),
        );
        frame.render_widget(
            &p,
            Rect::new(inner.x, inner.y + 4, " Ctrl-k ".len() as u16, 1),
        );
        frame.render_widget(
            &p,
            Rect::new(inner.x, inner.y + 6, " Ctrl-k ".len() as u16, 1),
        );
        Ok(())
    }
}
