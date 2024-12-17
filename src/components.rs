use color_eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    Frame,
};

use crate::{action::AppAction, app::Context};

pub mod actions_temp;
pub mod home;
pub mod popups;
pub mod results;
pub mod search;

// TODO: simple component for now
pub trait Component {
    fn update(&mut self, ctx: &Context, action: &AppAction) -> Result<Option<AppAction>>;
    fn on_key(&mut self, _ctx: &Context, _key: &KeyEvent) -> Result<()> {
        Ok(())
    }
    fn render(&mut self, ctx: &Context, frame: &mut Frame, area: Rect) -> Result<()>;
}

pub fn centered_rect<C: Into<Constraint>>(area: Rect, horizontal: C, vertical: C) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [center] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    center
}
