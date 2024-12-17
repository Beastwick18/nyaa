use color_eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph, Widget},
    Frame,
};

use crate::{
    action::AppAction,
    app::{Context, Mode},
};

use super::Component;

pub struct SearchComponent {
    content: String,
}

impl SearchComponent {
    pub fn new() -> Self {
        Self {
            content: String::new(),
        }
    }
}

impl Component for SearchComponent {
    fn update(
        &mut self,
        _ctx: &Context,
        _action: &AppAction,
    ) -> color_eyre::eyre::Result<Option<AppAction>> {
        Ok(None)
    }

    fn on_key(&mut self, ctx: &Context, _key: &KeyEvent) -> Result<()> {
        if ctx.mode != Mode::Home {
            return Ok(());
        }
        Ok(())
    }

    fn render(&mut self, _ctx: &Context, frame: &mut Frame, area: Rect) -> Result<()> {
        let block = Block::new().borders(Borders::ALL);
        Paragraph::new(&*self.content)
            .block(block)
            .render(area, frame.buffer_mut());
        Ok(())
    }
}
