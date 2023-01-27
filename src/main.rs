use std::process::{Command, Stdio};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use tui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

use crate::app::{App, InputMode};

mod app;
mod logging;
mod nyaa;
mod ui;

async fn search_nyaa(app: &mut App) {
    app.input_mode = InputMode::Normal;
    app.items.items.clear();

    let feed = nyaa::get_feed_list(&app.input, &app.category, &app.filter).await;
    // feed.sort_by(|a, b| b.downloads.cmp(&a.downloads));
    app.items.items = feed;
    app.items.select(0);
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    app.categories.select(0);
    app.filters.select(0);
    loop {
        terminal.draw(|f| ui::ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match app.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('h') => {}
                    KeyCode::Char('j') | KeyCode::Down => app.items.next(1),
                    KeyCode::Char('k') | KeyCode::Up => app.items.previous(1),
                    KeyCode::Char('J') => app.items.next(4),
                    KeyCode::Char('K') => app.items.previous(4),
                    KeyCode::Char('g') => {
                        app.items.select(0);
                    }
                    KeyCode::Char('G') => {
                        app.items.select(app.items.items.len() - 1);
                    }
                    KeyCode::Char('l') | KeyCode::Enter => {
                        if let Some(i) = app.items.state.selected() {
                            if let Some(item) = app.items.items.get(i) {
                                let _ = Command::new(
                                    "/mnt/c/Users/Brad/AppData/Local/WebTorrent/WebTorrent.exe",
                                )
                                .args([item.torrent_link.clone()])
                                .stdin(Stdio::null())
                                .stderr(Stdio::null())
                                .spawn();
                            }
                        }
                    }
                    KeyCode::Char('c') => {
                        app.last_input_mode = app.input_mode.to_owned();
                        app.input_mode = InputMode::SelectCategory;
                        // app.categories.select(0);
                    }
                    KeyCode::Char('f') => {
                        app.last_input_mode = app.input_mode.to_owned();
                        app.input_mode = InputMode::SelectFilter;
                    }
                    KeyCode::Char('/') => {
                        app.input_mode = InputMode::Editing;
                    }
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    _ => {}
                },
                InputMode::Editing => match key.code {
                    KeyCode::Enter => {
                        search_nyaa(&mut app).await;
                    }
                    KeyCode::Char(c) => {
                        app.input.push(c);
                    }
                    KeyCode::Backspace => {
                        app.input.pop();
                    }
                    KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
                    }
                    _ => {}
                },
                InputMode::SelectCategory => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        app.input_mode = app.last_input_mode.to_owned()
                    }
                    KeyCode::Char('/') => app.input_mode = InputMode::Editing,
                    KeyCode::Char('j') | KeyCode::Down => app.categories.next(1),
                    KeyCode::Char('k') | KeyCode::Up => app.categories.previous(1),
                    KeyCode::Enter | KeyCode::Char('l') => {
                        if let Some(i) = app.categories.state.selected() {
                            if let Some(item) = app.categories.items.get(i) {
                                app.category = item.to_owned().to_owned();
                                app.input_mode = app.last_input_mode.to_owned();
                                search_nyaa(&mut app).await;
                            }
                        }
                    }
                    _ => {}
                },
                InputMode::SelectFilter => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        app.input_mode = app.last_input_mode.to_owned()
                    }
                    KeyCode::Char('/') => app.input_mode = InputMode::Editing,
                    KeyCode::Char('j') | KeyCode::Down => app.filters.next(1),
                    KeyCode::Char('k') | KeyCode::Up => app.filters.previous(1),
                    KeyCode::Enter | KeyCode::Char('l') => {
                        if let Some(i) = app.filters.state.selected() {
                            if let Some(item) = app.filters.items.get(i) {
                                app.filter = item.to_owned().to_owned();
                                app.input_mode = app.last_input_mode.to_owned();
                                search_nyaa(&mut app).await;
                            }
                        }
                    }
                    _ => {}
                },
            }
        }
    }
}

#[tokio::main()]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = App::default();
    let _ = run_app(&mut terminal, app).await;

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
