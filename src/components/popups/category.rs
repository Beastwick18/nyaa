use color_eyre::Result;
use ratatui::{layout::Rect, Frame};

use crate::{action::AppAction, app::Context, components::Component};

struct Category {}

impl Component for Category {
    fn update(
        &mut self,
        _ctx: &Context,
        _action: &AppAction,
    ) -> Result<Option<crate::action::AppAction>> {
        Ok(None)
    }

    fn render(&mut self, _ctx: &Context, _frame: &mut Frame, _area: Rect) -> Result<()> {
        Ok(())
    }
}
