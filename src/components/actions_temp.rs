use color_eyre::Result;
use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::{action::AppAction, app::Context};

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
    fn update(
        &mut self,
        _ctx: &Context,
        action: &AppAction,
    ) -> Result<Option<crate::action::AppAction>> {
        if let AppAction::UserAction(m) = action {
            self.actions.push(format!("{m:?}"));
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
