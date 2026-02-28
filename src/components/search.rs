use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEventKind};
use ratatui::{
    Frame,
    layout::{Margin, Position, Rect},
    style::{Color, Stylize},
    widgets::{Block, Borders, Paragraph, Widget},
};
use tokio::sync::mpsc::UnboundedSender;
use tui_input::{Input, InputRequest};

use crate::{
    action::{AppAction, UserAction},
    app::{Context, InputMode, Mode},
    color::ColorRgbExt,
    keys::KeyComboStatus,
};

use super::Component;

pub struct SearchComponent {
    input: Input,
    last_area: Rect,
}

impl SearchComponent {
    pub fn new() -> Self {
        Self {
            input: Input::default(),
            last_area: Rect::default(),
        }
    }
}

impl Component for SearchComponent {
    fn update(
        &mut self,
        _ctx: &Context,
        action: &AppAction,
        action_tx: UnboundedSender<AppAction>,
    ) -> Result<()> {
        match action {
            AppAction::UserAction(UserAction::Insert(insert_action)) => {
                self.input.handle(*insert_action);
            }
            AppAction::UserAction(UserAction::Submit) => {
                action_tx.send(AppAction::Search)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn on_key(
        &mut self,
        ctx: &Context,
        key: &KeyEvent,
        action_tx: UnboundedSender<AppAction>,
    ) -> Result<()> {
        if ctx.mode == Mode::Search
            && ctx.input_mode == InputMode::Insert
            && ctx.keycombo.status() == &KeyComboStatus::Inserted
            && let KeyCode::Char(c) = key.code
        {
            self.input.handle(InputRequest::InsertChar(c));
            action_tx.send(AppAction::SetSearch(self.input.value().to_string()))?;
        }
        Ok(())
    }

    fn on_mouse(
        &mut self,
        ctx: &Context,
        event: &crossterm::event::MouseEvent,
        action_tx: UnboundedSender<AppAction>,
    ) -> Result<()> {
        let pos = (event.column, event.row).into();
        if event.kind == MouseEventKind::Down(MouseButton::Left)
            && self.last_area.contains(pos)
            && ctx.mode == Mode::Home
        {
            action_tx.send(AppAction::UserAction(UserAction::SetMode(Mode::Search)))?;
        }

        if event.kind == MouseEventKind::Drag(MouseButton::Left)
            && matches!(ctx.mode, Mode::Search | Mode::Home)
            && self.last_area.inner(Margin::new(1, 1)).contains(pos)
        {
            let x = event
                .column
                .saturating_sub(1)
                .saturating_sub(self.last_area.x);
            self.input.handle(InputRequest::SetCursor(x as usize));
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
        let block = Block::new().fg(bg.to_rgb()).borders(Borders::ALL);
        let (value, color) = if self.input.value().is_empty() {
            ("Search...", Color::Gray.to_rgb())
        } else {
            (self.input.value(), Color::White.to_rgb())
        };
        Paragraph::new(value)
            .block(block)
            .fg(color)
            .render(area, frame.buffer_mut());

        self.last_area = area;
        Ok(())
    }
}
