use color_eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Stylize},
    widgets::{Block, Widget as _},
    Frame,
};

use crate::{
    action::{AppAction, UserAction},
    app::Context,
};

use super::{
    actions_temp::ActionsComponent, results::ResultsComponent, search::SearchComponent, Component,
};

pub struct HomeComponent {
    search_size: u16,
    search: SearchComponent,
    results: ResultsComponent,
    actions_temp: ActionsComponent,
}

impl HomeComponent {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            search_size: 3,
            search: SearchComponent::new(),
            results: ResultsComponent::new(),
            actions_temp: ActionsComponent::new(),
        })
    }
}

impl Component for HomeComponent {
    fn update(&mut self, ctx: &Context, action: &AppAction) -> Result<Option<AppAction>> {
        self.results.update(ctx, action)?;
        self.actions_temp.update(ctx, action)?;

        if let AppAction::UserAction(UserAction::Submit) = action {
            return Ok(Some(AppAction::Search("queriees".to_string())));
        }

        Ok(None)
    }

    fn on_key(&mut self, ctx: &Context, key: &KeyEvent) -> Result<()> {
        self.results.on_key(ctx, key)?;
        Ok(())
    }

    fn render(&mut self, ctx: &Context, frame: &mut Frame, area: Rect) -> Result<()> {
        Block::new()
            .bg(Color::Rgb(34, 36, 54))
            .render(area, frame.buffer_mut());

        let vlayout = Layout::vertical([Constraint::Length(self.search_size), Constraint::Fill(1)])
            .split(area);
        let hlayout = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(vlayout[1]);
        self.search.render(ctx, frame, vlayout[0])?;
        self.results.render(ctx, frame, hlayout[0])?;
        self.actions_temp.render(ctx, frame, hlayout[1])?;

        Ok(())
    }
}
