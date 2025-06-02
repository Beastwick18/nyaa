use std::{path::PathBuf, time::Instant};

use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use enum_assoc::Assoc;
use serde::Deserialize;
use strum::{Display, EnumIter};
use tokio::sync::mpsc;

use crate::{
    action::{AppAction, TaskAction, UserAction},
    cli::Args,
    components::{home::HomeComponent, popups::PopupsComponent, Component},
    config::Config,
    keys::{self, KeyCombo, KeyComboStatus},
    result::ResultTable,
    sources::{Source, SourceTaskRunner},
    tui::{Tui, TuiEvent},
};

pub struct Context {
    pub config: Config,
    pub mode: Mode,
    pub input_mode: InputMode,
    pub keycombo: KeyCombo,
    pub source: Source,
    pub results: Option<ResultTable>,
    pub render_delta_time: f64,
}

impl Context {
    fn new(config_path: PathBuf) -> Result<Self> {
        Ok(Self {
            config: Config::new(config_path)?,
            mode: Mode::default(),
            input_mode: Mode::default().default_input_mode(),
            keycombo: KeyCombo::default(),
            source: Source::Nyaa,
            results: None,
            render_delta_time: 1.0 / 60.0,
        })
    }
}

#[derive(
    Deserialize, Default, Display, Debug, Hash, PartialEq, Eq, Copy, Clone, Assoc, EnumIter,
)]
#[func(pub fn input_modes(&self) -> Vec<InputMode> { vec![InputMode::Normal] })]
#[func(pub fn default_input_mode(&self) -> InputMode { InputMode::Normal })]
pub enum Mode {
    #[default]
    Home,
    DownloadClient,
    Categories,
    #[assoc(input_modes = vec![InputMode::Insert], default_input_mode = InputMode::Insert)]
    Search,
}

#[derive(Deserialize, Default, Display, Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub enum InputMode {
    #[default]
    Normal,
    Insert,
}

// impl Mode {
//     pub fn get_input_mode(&self) -> InputMode {
//         match self {
//             Self::Home => InputMode::Normal,
//             Self::DownloadClient => InputMode::Normal,
//             Self::Search => InputMode::Insert,
//         }
//     }
// }

pub struct App {
    ctx: Context,
    should_quit: bool,
    should_suspend: bool,
    action_tx: mpsc::UnboundedSender<AppAction>,
    action_rx: mpsc::UnboundedReceiver<AppAction>,
    components: Vec<Box<dyn Component>>,
    last_render_time: Instant,
    last_update_time: Instant,
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
            components: vec![HomeComponent::new(), PopupsComponent::new()],
            last_render_time: Instant::now(),
            last_update_time: Instant::now(),
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut tui = Tui::new()?
            .tick_rate(60.0) // TODO: Eliminate or gather from config
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
            TuiEvent::Tick => action_tx.send(AppAction::Tick)?,
            TuiEvent::Render => action_tx.send(AppAction::Render)?,
            TuiEvent::Resize(x, y) => action_tx.send(AppAction::Resize(x, y))?,
            TuiEvent::Key(key) => self.handle_key_event(key)?,
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        let action_tx = self.action_tx.clone();

        // Check for keys that may not be used in combos (resets current key combo).
        // Do not cancel if keycombo is *only* the cancelling key
        if self.ctx.keycombo.status() == &KeyComboStatus::Pending
            && keys::NON_COMBO.contains(&key.code)
        {
            self.ctx.keycombo.push_key(key);
            self.ctx.keycombo.set_status(KeyComboStatus::Cancelled);
        } else if self.ctx.input_mode == InputMode::Insert
            && key.modifiers == KeyModifiers::NONE
            && matches!(key.code, KeyCode::Char(c) if c.is_ascii_digit())
        {
            self.ctx.keycombo.set_status(KeyComboStatus::Inserted);
            self.ctx.keycombo.clear();
            self.ctx.keycombo.push_key(key);
        } else {
            // If no pending keycombo, reset to now be pending
            if self.ctx.keycombo.status() != &KeyComboStatus::Pending {
                self.ctx.keycombo.set_status(KeyComboStatus::Pending);
                self.ctx.keycombo.clear();
            }

            self.ctx.keycombo.push_key(key);

            let action = self.ctx.config.keys.action(
                self.ctx.keycombo.events(),
                &self.ctx.mode,
                &self.ctx.input_mode,
            );

            if let Some(action) = action {
                let mult = action
                    .multiplier()
                    .saturating_mul(self.ctx.keycombo.repeat().unwrap_or(1));

                let actions_to_send: Vec<AppAction> = action
                    .actions()
                    .into_iter()
                    .map(AppAction::UserAction)
                    .collect();

                for _ in 0..mult {
                    for action in actions_to_send.clone() {
                        action_tx.send(action)?
                    }
                }
                self.ctx.keycombo.set_status(KeyComboStatus::Successful);
            } else if self
                .ctx
                .config
                .keys
                .possible_actions(
                    self.ctx.keycombo.events(),
                    &self.ctx.mode,
                    &self.ctx.input_mode,
                )
                .next()
                .is_none()
            {
                if self.ctx.input_mode == InputMode::Insert {
                    self.ctx.keycombo.set_status(KeyComboStatus::Inserted);
                } else {
                    // No possible action with current keycombo
                    self.ctx.keycombo.set_status(KeyComboStatus::Unmatched);
                }
            }
        }

        for component in self.components.iter_mut() {
            component.on_key(&self.ctx, &key)?;
        }
        Ok(())
    }

    fn handle_actions(&mut self, tui: &mut Tui) -> Result<()> {
        while let Ok(action) = self.action_rx.try_recv() {
            match &action {
                AppAction::UserAction(u) => match u {
                    UserAction::Quit => self.should_quit = true,
                    UserAction::Suspend => self.should_suspend = true,
                    UserAction::SetMode(m) => {
                        self.ctx.mode = *m;
                        self.ctx.input_mode = self.ctx.mode.default_input_mode();
                    }
                    _ => {}
                },
                AppAction::Search(query) => self.search(query.clone()),
                AppAction::Resume => self.should_suspend = false,
                AppAction::Render => {
                    let elapsed = self.last_render_time.elapsed().as_secs_f64();
                    self.ctx.render_delta_time = elapsed;
                    self.last_render_time = Instant::now();
                    self.render(tui)?
                }
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

    fn search(&self, query: String) {
        let action_tx = self.action_tx.clone();
        let source = self.ctx.source;

        tokio::spawn(async move {
            let results = SourceTaskRunner::run(source, query).await;
            let _ = action_tx.send(AppAction::Task(TaskAction::SourceResults(results)));
        });
    }

    fn render(&mut self, tui: &mut Tui) -> Result<()> {
        if self.ctx.input_mode == InputMode::Insert {
            tui.show_cursor()?;
        } else {
            tui.hide_cursor()?;
        }

        tui.draw(|frame| {
            for component in self.components.iter_mut() {
                if let Err(err) = component.render(&self.ctx, frame, frame.area()) {
                    let _ = self
                        .action_tx
                        .send(AppAction::Error(format!("Failed to draw: {:?}", err)));
                }
            }
        })?;
        Ok(())
    }
}
