use color_eyre::Result;
use crossterm::event::KeyEvent;
use download_client::DownloadClientComponent;
use ratatui::{layout::Rect, Frame};
use which_key::WhichKeyComponent;

use crate::{action::AppAction, app::Context};

use super::Component;

pub mod download_client;
pub mod which_key;

pub struct PopupsComponent {
    popups: Vec<Box<dyn Component>>,
}

impl PopupsComponent {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            popups: vec![DownloadClientComponent::new(), WhichKeyComponent::new()],
        })
    }
}

impl Component for PopupsComponent {
    fn update(&mut self, ctx: &Context, action: &AppAction) -> Result<Option<AppAction>> {
        for popup in self.popups.iter_mut() {
            popup.update(ctx, action)?;
        }
        Ok(None)
    }

    fn on_key(&mut self, ctx: &Context, key: &KeyEvent) -> Result<()> {
        for popup in self.popups.iter_mut() {
            popup.on_key(ctx, key)?;
        }
        Ok(())
    }

    fn render(&mut self, ctx: &Context, frame: &mut Frame, area: Rect) -> Result<()> {
        for popup in self.popups.iter_mut() {
            popup.render(ctx, frame, area)?;
        }
        Ok(())
    }
}
