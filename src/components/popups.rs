use color_eyre::Result;
use crossterm::event::KeyEvent;
use download_client::DownloadClientComponent;
use ratatui::{layout::Rect, Frame};

use crate::{action::AppAction, app::Context};

use super::Component;

pub mod download_client;

pub struct PopupsComponent {
    // popups: IndexMap<Mode, Box<dyn Component>>,
    popups: Vec<Box<dyn Component>>,
}

impl PopupsComponent {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            // popups: [(
            //     Mode::DownloadClient,
            //     DownloadClientComponent::new() as Box<dyn Component>,
            // )]
            popups: vec![DownloadClientComponent::new()],
        })
    }
}

impl Component for PopupsComponent {
    fn update(&mut self, ctx: &Context, action: &AppAction) -> Result<Option<AppAction>> {
        // if let Some(popup) = self.popups.get_mut(&ctx.mode) {
        //     return popup.update(ctx, action);
        // }
        for popup in self.popups.iter_mut() {
            popup.update(ctx, action)?;
        }
        Ok(None)
    }

    fn on_key(&mut self, ctx: &Context, key: &KeyEvent) -> Result<()> {
        // if let Some(popup) = self.popups.get_mut(&ctx.mode) {
        //     return popup.on_key(ctx, key);
        // }
        for popup in self.popups.iter_mut() {
            popup.on_key(ctx, key)?;
        }
        Ok(())
    }

    fn render(&mut self, ctx: &Context, frame: &mut Frame, area: Rect) -> Result<()> {
        // if let Some(popup) = self.popups.get_mut(&ctx.mode) {
        //     return popup.render(ctx, frame, area);
        // }
        for popup in self.popups.iter_mut() {
            popup.render(ctx, frame, area)?;
        }

        Ok(())
    }
}
