use color_eyre::Result;
use crossterm::event::MouseEvent;
use ratatui::{
    layout::Rect,
    style::{Color, Stylize as _},
    widgets::{Block, Borders, Paragraph, Widget as _, Wrap},
    Frame,
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    action::AppAction,
    animate::{translate::Translate, Animation, AnimationState, Direction, Smoothing},
    app::{Context, Mode},
    color::ColorRgbExt,
    components,
    mouse::drag::{DragEdge, DragExt, DragState},
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
    drag: DragState,
}

impl Categories {
    pub fn boxed() -> Box<dyn Component> {
        Box::new(Self {
            translate_state: AnimationState::new(6.0)
                .playing(true)
                .backwards()
                .smoothing(Smoothing::EaseInAndOut),
            drag: DragState::edge(DragEdge::ALL),
        })
    }
}

impl Component for Categories {
    fn update(
        &mut self,
        ctx: &Context,
        action: &AppAction,
        _action_tx: UnboundedSender<AppAction>,
    ) -> Result<()> {
        if action == &AppAction::Render {
            self.translate_state.set_direction(match ctx.mode {
                Mode::Categories => Direction::Forwards,
                _ => Direction::Backwards,
            });

            self.translate_state.update(ctx.render_delta_time);
        }

        Ok(())
    }

    fn on_mouse(
        &mut self,
        ctx: &Context,
        event: &MouseEvent,
        _action_tx: UnboundedSender<AppAction>,
    ) -> Result<()> {
        if ctx.mode != Mode::Categories {
            return Ok(());
        }

        self.drag.on_mouse(event);

        Ok(())
    }

    fn render(&mut self, _ctx: &Context, frame: &mut Frame, area: Rect) -> Result<()> {
        let center = components::centered_rect(area, 50, 10).drag(&self.drag, area);
        // let center = self.drag.drag(center, area);

        let mut center_bottom = components::centered_rect(area, 50, 10);
        center_bottom.y = area.height + area.y;

        ClearOverlap.render(center, frame.buffer_mut());

        let bg = Block::new().bg(Color::Rgb(0, 36, 54)).borders(Borders::ALL);
        let p = Paragraph::new("Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.")
            .fg(Color::White.to_rgb())
            .block(bg)
            .wrap(Wrap { trim: false });

        let translate = Translate::new(&self.translate_state, center_bottom.into(), center.into());
        self.drag.set_last_area(translate.area().into());
        translate.render_widget(p, area, frame.buffer_mut());

        Ok(())
    }
}
