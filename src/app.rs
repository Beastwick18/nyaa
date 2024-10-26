use std::path::PathBuf;

use color_eyre::Result;
use crossterm::event::KeyEvent;
use serde::Deserialize;
use strum::Display;
use tokio::sync::mpsc;
use tracing::debug;

use crate::{
    action::{AppAction, UserAction},
    cli::Args,
    components::{home::HomeComponent, Component},
    config::Config,
    tui::{Tui, TuiEvent},
};

pub struct Context {
    pub config: Config,
    pub mode: Mode,
}

impl Context {
    fn new(config_path: PathBuf) -> Result<Self> {
        Ok(Self {
            config: Config::new(config_path)?,
            mode: Mode::default(),
        })
    }
}

#[derive(Deserialize, Default, Display, Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub enum Mode {
    #[default]
    Home,
    Test, // TODO: Remove
}

pub struct App {
    // config: Config,
    // mode: Mode,
    ctx: Context,
    should_quit: bool,
    should_suspend: bool,
    action_tx: mpsc::UnboundedSender<AppAction>,
    action_rx: mpsc::UnboundedReceiver<AppAction>,
    components: Vec<Box<dyn Component>>,
}

impl App {
    pub fn new(args: Args) -> Result<Self> {
        let (action_tx, action_rx) = mpsc::unbounded_channel();
        let config_path = args
            .config_path
            .map(Into::into)
            .unwrap_or_else(Config::default_config_path);
        Ok(Self {
            ctx: Context::new(config_path)?,
            should_quit: false,
            should_suspend: false,
            action_tx,
            action_rx,
            components: vec![HomeComponent::new()],
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut tui = Tui::new()?
            .tick_rate(4.0) // TODO: Eliminate or gather from config
            .frame_rate(60.0); // TODO: Eliminate or gather from config
        tui.enter()?;

        // Initialize components

        let action_tx = self.action_tx.clone();
        loop {
            self.handle_events(&mut tui).await?;
            self.handle_actions(&mut tui)?;

            if self.should_suspend {
                tui.suspend()?;
                action_tx.send(AppAction::Resume)?;
                action_tx.send(AppAction::ClearScreen)?;
                tui.enter()?;
            } else if self.should_quit {
                tui.stop()?;
                break;
            }
        }

        tui.exit()?;

        Ok(())
    }

    async fn handle_events(&mut self, tui: &mut Tui) -> Result<()> {
        let Some(event) = tui.next_event().await else {
            return Ok(());
        };

        let action_tx = self.action_tx.clone();
        match event {
            // TuiEvent::Quit => action_tx.send(AppAction::UserAction(UserAction::Quit))?,
            TuiEvent::Tick => action_tx.send(AppAction::Tick)?,
            TuiEvent::Render => action_tx.send(AppAction::Render)?,
            TuiEvent::Resize(x, y) => action_tx.send(AppAction::Resize(x, y))?,
            TuiEvent::Key(key) => self.handle_key_event(key)?,
            _ => {}
        };
        // for component in self.components.iter_mut() {
        //     if let Some(action) = components.handle_events(Some(event.clone()))? {
        //         action_tx.send(action)?;
        //     }
        // }
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        let action_tx = self.action_tx.clone();

        let Some(keymap) = self.ctx.config.keys.get(&self.ctx.mode) else {
            return Ok(());
        };
        match keymap.get(&vec![key]) {
            Some(action) => {
                action_tx.send(AppAction::UserAction(action.clone()))?;
            }
            _ => {
                // TODO: Handle multikey
            }
        }
        for component in self.components.iter_mut() {
            component.on_key(&self.ctx, &key)?;
        }
        Ok(())
    }

    fn handle_actions(&mut self, tui: &mut Tui) -> Result<()> {
        while let Ok(action) = self.action_rx.try_recv() {
            if action != AppAction::Tick && action != AppAction::Render {
                // Special action
                debug!("{action:?}");
            }
            match &action {
                AppAction::UserAction(u) => match u {
                    UserAction::Quit => self.should_quit = true,
                    UserAction::Suspend => self.should_suspend = true,
                    UserAction::SetMode(m) => self.ctx.mode = *m,
                    _ => {}
                },
                AppAction::Resume => self.should_suspend = false,
                AppAction::Render => self.render(tui)?,
                AppAction::ClearScreen => tui.clear()?,
                _ => {}
            }
            for component in self.components.iter_mut() {
                if let Some(action) = component.update(&self.ctx, &action)? {
                    self.action_tx.send(action)?;
                }
            }
        }
        Ok(())
    }

    fn render(&mut self, tui: &mut Tui) -> Result<()> {
        tui.draw(|frame| {
            for component in self.components.iter_mut() {
                if let Err(err) = component.render(frame, frame.area()) {
                    let _ = self
                        .action_tx
                        .send(AppAction::Error(format!("Failed to draw: {:?}", err)));
                }
            }
        })?;
        Ok(())
    }
}
