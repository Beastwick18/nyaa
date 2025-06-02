use color_eyre::Result;
use ratatui::{
    layout::Rect,
    style::{Color, Stylize as _},
    widgets::{Block, Paragraph, Widget as _, Wrap},
    Frame,
};

use crate::{
    action::AppAction,
    animate::{translate::Translate, Animation, AnimationState, Direction, Smoothing},
    app::{Context, Mode},
    components,
    widgets::clear_overlap::ClearOverlap,
};

use super::Component;

// TODO: To be defined by each source
pub enum CategoriesTest {
    AllCategories,
    Anime,
    Music,
    Games,
}

pub enum AnimeCategory {}

pub struct Categories {
    translate_state: AnimationState,
    // grow_state: GrowthAnimation,
}

impl Categories {
    pub fn boxed() -> Box<dyn Component> {
        Box::new(Self {
            translate_state: AnimationState::new(6.0)
                .playing(true)
                .backwards()
                .smoothing(Smoothing::EaseInAndOut),
        })
    }
}

impl Component for Categories {
    fn update(&mut self, ctx: &Context, action: &AppAction) -> Result<Option<AppAction>> {
        if action == &AppAction::Render {
            self.translate_state.set_direction(match ctx.mode {
                Mode::Categories => Direction::Forwards,
                _ => Direction::Backwards,
            });

            self.translate_state.update(ctx.render_delta_time);
        }

        Ok(None)
    }

    fn render(&mut self, _ctx: &Context, frame: &mut Frame, area: Rect) -> Result<()> {
        let center = components::centered_rect(area, 50, 10);
        let mut center_bottom = components::centered_rect(area, 50, 10);
        center_bottom.y = area.height + area.y;

        ClearOverlap.render(center, frame.buffer_mut());

        let bg = Block::new().bg(Color::Rgb(0, 36, 54));
        let p = Paragraph::new("Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.")
            .fg(Color::White)
            .block(bg)
            .wrap(Wrap { trim: false });

        Translate::new(&self.translate_state, center_bottom.into(), center.into()).render_widget(
            p,
            area,
            frame.buffer_mut(),
        );

        Ok(())
    }
}
