use color_eyre::Result;
use ratatui::{layout::Rect, Frame};

use crate::action::AppAction;

pub mod actions_temp;
pub mod home;
pub mod results;

// TODO: simple component for now
pub trait Component {
    fn update(&mut self, action: &AppAction) -> Result<Option<AppAction>>;
    fn render(&mut self, frame: &mut Frame, area: Rect) -> Result<()>;
}
