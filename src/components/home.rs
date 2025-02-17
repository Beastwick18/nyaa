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
    animate::{AnimationState, Direction, Smoothing},
    app::{Context, Mode},
    widgets::dim::Dim,
};

use super::{
    actions_temp::ActionsComponent, results::ResultsComponent, search::SearchComponent, Component,
};

pub struct HomeComponent {
    search_size: u16,
    search: SearchComponent,
    results: ResultsComponent,
    actions_temp: ActionsComponent,
    dim_state: AnimationState,
}

impl HomeComponent {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            search_size: 3,
            search: SearchComponent::new(),
            results: ResultsComponent::new(),
            actions_temp: ActionsComponent::new(),
            dim_state: AnimationState::new(0.04)
                .playing(true)
                .smoothing(Smoothing::EaseInAndOut)
                .backwards(),
        })
    }
}

impl Component for HomeComponent {
    fn update(&mut self, ctx: &Context, action: &AppAction) -> Result<Option<AppAction>> {
        self.results.update(ctx, action)?;
        self.actions_temp.update(ctx, action)?;

        self.dim_state.set_direction(match ctx.mode {
            Mode::Home => Direction::Backwards,
            _ => Direction::Forwards,
        });
        if let AppAction::Tick = action {
            self.dim_state.update();
        }
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

        Dim::new(0.5)
            .animated(self.dim_state)
            .render(area, frame.buffer_mut());

        Ok(())
    }
}
