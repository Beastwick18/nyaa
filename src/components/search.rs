use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Position, Rect},
    style::{Color, Stylize},
    widgets::{Block, Borders, Paragraph, Widget},
    Frame,
};
use tui_input::{Input, InputRequest};

use crate::{
    action::{AppAction, UserAction},
    app::{Context, InputMode, Mode},
    color::to_rgb,
    keys::KeyComboStatus,
};

use super::Component;

pub struct SearchComponent {
    input: Input,
}

impl SearchComponent {
    pub fn new() -> Self {
        Self {
            input: Input::default(),
        }
    }
}

impl Component for SearchComponent {
    fn update(&mut self, _ctx: &Context, action: &AppAction) -> Result<Option<AppAction>> {
        if let AppAction::UserAction(UserAction::Insert(insert_action)) = action {
            self.input.handle(*insert_action);
        }
        Ok(None)
    }

    fn on_key(&mut self, ctx: &Context, key: &KeyEvent) -> Result<()> {
        if ctx.mode == Mode::Search
            && ctx.input_mode == InputMode::Insert
            && ctx.keycombo.status() == &KeyComboStatus::Inserted
        {
            if let KeyCode::Char(c) = key.code {
                self.input.handle(InputRequest::InsertChar(c));
            }
        }
        Ok(())
    }

    fn render(&mut self, ctx: &Context, frame: &mut Frame, area: Rect) -> Result<()> {
        if ctx.mode == Mode::Search {
            frame.set_cursor_position(Position::new(
                area.x + 1 + self.input.visual_cursor() as u16,
                area.y + 1,
            ));
        }
        let bg = match ctx.mode {
            Mode::Search => Color::Cyan,
            _ => Color::White,
        };
        let block = Block::new().fg(to_rgb(bg)).borders(Borders::ALL);
        Paragraph::new(self.input.value())
            .block(block)
            .fg(Color::Rgb(255, 255, 255))
            .render(area, frame.buffer_mut());
        Ok(())
    }
}
