use color_eyre::Result;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    Frame,
};

use crate::action::AppAction;

use super::{actions_temp::ActionsComponent, results::ResultsComponent, Component};

pub struct HomeComponent {
    results: ResultsComponent,
    actions_temp: ActionsComponent,
    // batch: BatchComponent,
    // search: SearchComponent,
}

impl HomeComponent {
    pub fn new() -> Self {
        Self {
            results: ResultsComponent::new(),
            actions_temp: ActionsComponent::new(),
        }
    }
}

impl Component for HomeComponent {
    fn update(&mut self, action: &AppAction) -> Result<Option<AppAction>> {
        self.results.update(action)?;
        self.actions_temp.update(action)?;
        Ok(None)
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let layout = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);
        self.results.render(frame, layout[0])?;
        self.actions_temp.render(frame, layout[1])?;

        Ok(())
    }
}
