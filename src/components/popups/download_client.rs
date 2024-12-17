use color_eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph, Widget},
    Frame,
};

use crate::{
    action::AppAction,
    animate::{Animation, AnimationType},
    app::{Context, Mode},
    components,
    widgets::clear_overlap::ClearOverlap,
};

use super::Component;

pub struct DownloadClientComponent {
    animation: Animation,
}

impl DownloadClientComponent {
    pub fn new() -> Box<DownloadClientComponent> {
        Box::new(Self {
            animation: Animation::new(AnimationType::EaseInAndOut, 0.04)
                .playing()
                .reversed(),
        })
    }
}

impl Component for DownloadClientComponent {
    fn update(&mut self, ctx: &Context, _action: &AppAction) -> Result<Option<AppAction>> {
        if ctx.mode == Mode::DownloadClient {
            self.animation.forwards();
        } else {
            self.animation.backwards();
        }
        self.animation.update();
        Ok(None)
    }

    fn on_key(&mut self, _ctx: &Context, _key: &KeyEvent) -> Result<()> {
        Ok(())
    }

    fn render(&mut self, _ctx: &Context, frame: &mut Frame, area: Rect) -> Result<()> {
        let center = components::centered_rect(area, 100, 10);
        let mut center_bottom = components::centered_rect(area, 100, 10);
        center_bottom.y = area.height + area.y;

        let anim_area = self
            .animation
            .animate(center_bottom, center)
            .intersection(area);
        ClearOverlap::default().render(anim_area, frame.buffer_mut());
        Paragraph::new("Testing :3")
            .block(Block::new().borders(Borders::ALL))
            .render(anim_area, frame.buffer_mut());
        Ok(())
    }
}
