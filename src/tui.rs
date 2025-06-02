use std::io::{stdout, Stdout};
use std::ops::{Deref, DerefMut};
use std::time::Duration;

use color_eyre::Result;

use crossterm::cursor::{self, SetCursorStyle};
use crossterm::event::{
    DisableBracketedPaste, EnableBracketedPaste, Event, EventStream, KeyEvent, KeyEventKind,
    MouseEvent,
};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use futures::{FutureExt as _, StreamExt as _};
use ratatui::backend::CrosstermBackend as Backend;
use ratatui::Terminal;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::error;

pub struct Tui {
    pub terminal: Terminal<Backend<Stdout>>,
    pub event_task: Option<JoinHandle<()>>,
    pub cancel_token: CancellationToken,
    pub event_rx: UnboundedReceiver<TuiEvent>,
    pub event_tx: UnboundedSender<TuiEvent>,
    pub frame_rate: f64,
    pub tick_rate: f64,
    pub cursor_style: SetCursorStyle,
}

pub enum TuiEvent {
    Init, // TODO: determine if necessary
    FocusGained,
    FocusLost,
    // Quit,
    Tick,
    Render,
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    Paste(String),
    Error,
}

impl Tui {
    pub fn new() -> Result<Self> {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        Ok(Self {
            terminal: Terminal::new(Backend::new(stdout()))?,
            event_task: None,
            cancel_token: CancellationToken::new(),
            event_rx,
            event_tx,
            frame_rate: 60.0,
            tick_rate: 60.0,
            cursor_style: SetCursorStyle::BlinkingBar,
        })
    }

    pub fn tick_rate(mut self, tick_rate: f64) -> Self {
        self.tick_rate = tick_rate;
        self
    }

    pub fn frame_rate(mut self, frame_rate: f64) -> Self {
        self.frame_rate = frame_rate;
        self
    }

    pub fn start(&mut self) {
        self.cancel();
        self.cancel_token = CancellationToken::new();

        let event_loop = Self::event_loop(
            self.event_tx.clone(),
            self.cancel_token.clone(),
            self.tick_rate,
            self.frame_rate,
        );
        self.event_task = Some(tokio::spawn(async {
            event_loop.await;
        }));
    }

    pub fn stop(&mut self) -> Result<()> {
        self.cancel();
        let mut counter = 0;
        if let Some(event_task) = self.event_task.as_ref() {
            while !event_task.is_finished() {
                std::thread::sleep(Duration::from_millis(1));
                counter += 1;
                if counter > 50 {
                    event_task.abort();
                }
                if counter > 100 {
                    error!("Failed to abort task in 100ms");
                }
            }
        }
        Ok(())
    }

    async fn event_loop(
        event_tx: UnboundedSender<TuiEvent>,
        cancel_token: CancellationToken,
        tick_rate: f64,
        frame_rate: f64,
    ) {
        let mut event_stream = EventStream::new();
        let mut tick_interval = tokio::time::interval(Duration::from_secs_f64(1.0 / tick_rate));
        let mut frame_interval = tokio::time::interval(Duration::from_secs_f64(1.0 / frame_rate));

        event_tx
            .send(TuiEvent::Init)
            .expect("Failed to send initialize event");

        loop {
            let event = tokio::select! {
                _ = cancel_token.cancelled() => {
                    break; // Event loop cancelled
                }
                _ = tick_interval.tick() => TuiEvent::Tick,
                _ = frame_interval.tick() => TuiEvent::Render,
                crossterm_event = event_stream.next().fuse() => match crossterm_event {
                    Some(Ok(event)) => match event {
                        Event::Key(key) if key.kind == KeyEventKind::Press => TuiEvent::Key(key),
                        Event::Mouse(mouse) => TuiEvent::Mouse(mouse),
                        Event::Resize(x, y) => TuiEvent::Resize(x, y),
                        Event::FocusLost => TuiEvent::FocusLost,
                        Event::FocusGained => TuiEvent::FocusGained,
                        Event::Paste(s) => TuiEvent::Paste(s),
                        _ => continue,
                    }
                    Some(Err(_)) => TuiEvent::Error,
                    None => break, // Event stream terminated
                }
            };
            if event_tx.send(event).is_err() {
                break; // receiver dropped
            }
        }
        cancel_token.cancel();
    }

    pub fn cancel(&self) {
        self.cancel_token.cancel();
    }

    pub fn enter(&mut self) -> Result<()> {
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(
            stdout(),
            EnterAlternateScreen,
            EnableBracketedPaste,
            cursor::Hide,
            self.cursor_style
        )?;

        self.start();
        Ok(())
    }

    pub fn exit(&mut self) -> Result<()> {
        self.stop()?;
        if crossterm::terminal::is_raw_mode_enabled()? {
            self.flush()?;
            crossterm::execute!(
                stdout(),
                DisableBracketedPaste,
                LeaveAlternateScreen,
                cursor::Show,
                cursor::SetCursorStyle::DefaultUserShape
            )?;
            crossterm::terminal::disable_raw_mode()?;
        }
        Ok(())
    }

    // In event of panic, abort TUI session
    pub fn abort() {
        let _ = crossterm::execute!(
            stdout(),
            DisableBracketedPaste,
            LeaveAlternateScreen,
            cursor::Show
        );
        let _ = crossterm::terminal::disable_raw_mode();
    }

    pub fn suspend(&mut self) -> Result<()> {
        self.exit()?;
        #[cfg(not(windows))]
        signal_hook::low_level::raise(signal_hook::consts::signal::SIGTSTP)?;
        Ok(())
    }

    pub async fn next_event(&mut self) -> Option<TuiEvent> {
        self.event_rx.recv().await
    }

    pub fn hide_cursor(&mut self) -> Result<()> {
        self.terminal.hide_cursor()?;
        Ok(())
    }

    pub fn show_cursor(&mut self) -> Result<()> {
        self.terminal.show_cursor()?;
        Ok(())
    }
}

impl Deref for Tui {
    type Target = ratatui::Terminal<Backend<Stdout>>;

    fn deref(&self) -> &Self::Target {
        &self.terminal
    }
}

impl DerefMut for Tui {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.terminal
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        self.exit().unwrap();
    }
}
