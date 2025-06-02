use category::Categories;
use color_eyre::Result;
use crossterm::event::KeyEvent;
use download_client::DownloadClientComponent;
use notification::NotificationContainer;
use ratatui::{layout::Rect, widgets::Widget, Frame};
use which_key::WhichKeyComponent;

use crate::{
    action::AppAction,
    animate::{AnimationState, Direction, Smoothing},
    app::{Context, Mode},
    widgets::dim::Dim,
};

use super::Component;

pub mod category;
pub mod download_client;
pub mod filter;
pub mod notification;
pub mod which_key;

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub enum PopupMode {
    All,
    Some(Mode),
    // None,
}

pub struct PopupsComponent {
    popups: Vec<(PopupMode, Box<dyn Component>)>,
    dim_state: AnimationState,
}

impl PopupsComponent {
    pub fn new() -> Box<Self> {
        let popups: Vec<(PopupMode, Box<dyn Component>)> = vec![
            (PopupMode::All, NotificationContainer::boxed()),
            (
                PopupMode::Some(Mode::DownloadClient),
                DownloadClientComponent::boxed(),
            ),
            (PopupMode::Some(Mode::Categories), Categories::boxed()),
            (PopupMode::All, WhichKeyComponent::boxed()),
        ];
        Box::new(Self {
            popups,
            dim_state: AnimationState::new(4.0)
                .playing(true)
                .smoothing(Smoothing::EaseInAndOut)
                .backwards(),
        })
    }
}

impl Component for PopupsComponent {
    fn update(&mut self, ctx: &Context, action: &AppAction) -> Result<Option<AppAction>> {
        if &AppAction::Render == action {
            let direction = match self
                .popups
                .iter()
                .any(|(k, _)| k == &PopupMode::Some(ctx.mode))
            {
                true => Direction::Forwards,
                false => Direction::Backwards,
            };

            self.dim_state.set_direction(direction);

            // self.dim_state.set_direction(match ctx.mode {
            //     Mode::Home => Direction::Backwards,
            //     _ => Direction::Forwards,
            // });

            self.dim_state.update(ctx.render_delta_time);
        }

        for (_, popup) in self.popups.iter_mut() {
            popup.update(ctx, action)?;
        }
        Ok(None)
    }

    fn on_key(&mut self, ctx: &Context, key: &KeyEvent) -> Result<()> {
        for (_, popup) in self.popups.iter_mut() {
            popup.on_key(ctx, key)?;
        }
        Ok(())
    }

    fn render(&mut self, ctx: &Context, frame: &mut Frame, area: Rect) -> Result<()> {
        Dim::new(0.7)
            .animated(self.dim_state)
            .render(area, frame.buffer_mut());

        for (_, popup) in self.popups.iter_mut() {
            popup.render(ctx, frame, area)?;
        }
        Ok(())
    }
}
