use color_eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::{layout::Rect, Frame};

use crate::{action::AppAction, app::Context};

pub mod actions_temp;
pub mod home;
pub mod results;

// TODO: simple component for now
pub trait Component {
    fn update(&mut self, ctx: &Context, action: &AppAction) -> Result<Option<AppAction>>;
    fn on_key(&mut self, _ctx: &Context, _key: &KeyEvent) -> Result<()> {
        Ok(())
    }
    fn render(&mut self, frame: &mut Frame, area: Rect) -> Result<()>;
}
