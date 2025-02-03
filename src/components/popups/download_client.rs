use color_eyre::Result;
use ratatui::{
    layout::Rect,
    style::{Color, Stylize as _},
    widgets::{Block, Paragraph, Widget as _, Wrap},
    Frame,
};

use crate::{
    action::AppAction,
    animate::{
        growth::{Growth, GrowthAnimation},
        translate::Translate,
        Animation, AnimationState, Direction, Smoothing,
    },
    app::{Context, Mode},
    components,
    widgets::clear_overlap::ClearOverlap,
};

use super::Component;

pub struct DownloadClientComponent {
    translate: Translate,
    anim2: GrowthAnimation,
}

impl DownloadClientComponent {
    pub fn new() -> Box<DownloadClientComponent> {
        Box::new(Self {
            translate: AnimationState::new(0.06)
                .playing(true)
                .backwards()
                .smoothing(Smoothing::EaseInAndOut)
                .into(),
            anim2: GrowthAnimation::new(
                AnimationState::new(0.04)
                    .playing(true)
                    .smoothing(Smoothing::EaseInAndOut)
                    .backwards(),
                Growth::Random,
            ),
        })
    }
}

impl Component for DownloadClientComponent {
    fn update(&mut self, ctx: &Context, action: &AppAction) -> Result<Option<AppAction>> {
        self.anim2.state_mut().set_direction(match ctx.mode {
            Mode::DownloadClient => Direction::Forwards,
            _ => Direction::Backwards,
        });

        self.translate.state_mut().set_direction(match ctx.mode {
            Mode::DownloadClient => Direction::Forwards,
            _ => Direction::Backwards,
        });

        if let AppAction::Tick = action {
            self.anim2.state_mut().update();
            self.translate.state_mut().update();
        }

        Ok(None)
    }

    fn render(&mut self, _ctx: &Context, frame: &mut Frame, area: Rect) -> Result<()> {
        let center = components::centered_rect(area, 100, 10);
        let mut center_bottom = components::centered_rect(area, 100, 10);
        center_bottom.y = area.height + area.y;

        ClearOverlap.render(center, frame.buffer_mut());

        let bg = Block::new().bg(Color::Rgb(34, 36, 54));
        let p = Paragraph::new("Testing :3. This is a really long paragraph. I don't know what I want to say, however I will continue typing.")
            .fg(Color::White)
            .block(bg)
            .wrap(Wrap {trim: false});
        self.anim2.render_widget(p, center, frame.buffer_mut());
        // let anim = self
        //     .translate
        //     .start(center_bottom)
        //     .stop(center)
        //     .area()
        //     .intersection(area);
        // let inner = anim.inner(Margin {
        //     horizontal: 1,
        //     vertical: 1,
        // });
        // Clear.render(anim, frame.buffer_mut());
        // bg.render(anim, frame.buffer_mut());
        // p.render(inner, frame.buffer_mut());

        Ok(())
    }
}
