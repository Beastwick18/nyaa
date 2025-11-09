use color_eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Stylize},
    widgets::{Block, Widget as _},
    Frame,
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    action::AppAction,
    app::{Context, Mode},
};

use super::{results::ResultsComponent, search::SearchComponent, Component};

pub struct HomeComponent {
    search_size: u16,
    search: SearchComponent,
    results: ResultsComponent,
}

impl HomeComponent {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            search_size: 3,
            search: SearchComponent::new(),
            results: ResultsComponent::new(),
        })
    }
}

impl Component for HomeComponent {
    fn update(
        &mut self,
        ctx: &Context,
        action: &AppAction,
        action_tx: UnboundedSender<AppAction>,
    ) -> Result<()> {
        self.results.update(ctx, action, action_tx.clone())?;

        if ctx.mode == Mode::Search {
            return self.search.update(ctx, action, action_tx.clone());
        }

        Ok(())
    }

    fn on_key(&mut self, ctx: &Context, key: &KeyEvent) -> Result<()> {
        if ctx.mode == Mode::Search {
            self.search.on_key(ctx, key)?;
        }
        self.results.on_key(ctx, key)?;
        Ok(())
    }

    fn render(&mut self, ctx: &Context, frame: &mut Frame, area: Rect) -> Result<()> {
        Block::new()
            .bg(Color::Rgb(34, 36, 54))
            .render(area, frame.buffer_mut());

        let vlayout = Layout::vertical([Constraint::Length(self.search_size), Constraint::Fill(1)])
            .split(area);
        let hlayout = Layout::horizontal([Constraint::Percentage(100)]).split(vlayout[1]);
        self.search.render(ctx, frame, vlayout[0])?;
        self.results.render(ctx, frame, hlayout[0])?;

        Ok(())
    }
}
