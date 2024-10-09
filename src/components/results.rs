use color_eyre::Result;
use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::action::AppAction;

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
    fn update(&mut self, action: &AppAction) -> color_eyre::eyre::Result<Option<AppAction>> {
        if action == &AppAction::Resume {
            self.content = "Welcome back, user".to_string();
        }
        Ok(None)
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let p =
            Paragraph::new(format!("{}!", self.content)).block(Block::new().borders(Borders::ALL));
        frame.render_widget(p, area);
        Ok(())
    }
}
