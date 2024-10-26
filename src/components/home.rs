use color_eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    Frame,
};

use crate::{action::AppAction, app::Context};

use super::{actions_temp::ActionsComponent, results::ResultsComponent, Component};

pub struct HomeComponent {
    results: ResultsComponent,
    actions_temp: ActionsComponent,
    // batch: BatchComponent,
    // search: SearchComponent,
}

impl HomeComponent {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            results: ResultsComponent::new(),
            actions_temp: ActionsComponent::new(),
        })
    }
}

impl Component for HomeComponent {
    fn update(&mut self, ctx: &Context, action: &AppAction) -> Result<Option<AppAction>> {
        self.results.update(ctx, action)?;
        self.actions_temp.update(ctx, action)?;
        Ok(None)
    }

    fn on_key(&mut self, ctx: &Context, key: &KeyEvent) -> Result<()> {
        self.results.on_key(ctx, key)?;
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let layout = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);
        self.results.render(frame, layout[0])?;
        self.actions_temp.render(frame, layout[1])?;

        Ok(())
    }
}
