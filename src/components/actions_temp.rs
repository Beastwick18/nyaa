use color_eyre::Result;
use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::action::{AppAction, UserAction};

use super::Component;

pub struct ActionsComponent {
    actions: Vec<String>,
}

impl ActionsComponent {
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
        }
    }
}

impl Component for ActionsComponent {
    fn update(&mut self, action: &AppAction) -> Result<Option<crate::action::AppAction>> {
        if let AppAction::UserAction(UserAction::SetMode(m)) = action {
            self.actions.push(m.to_string());
        }
        Ok(None)
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let p = Paragraph::new(self.actions.join("\n"))
            .block(Block::new().borders(Borders::ALL).title("User Actions"));
        frame.render_widget(p, area);

        Ok(())
    }
}
