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
use queues::IsQueue;

use crate::app::{App, InputMode};
use crate::nyaa::Sort;

mod app;
mod logging;
mod nyaa;
mod ui;

async fn search_nyaa(app: &mut App) {
    app.input_mode = InputMode::Normal;
    app.items.items.clear();

    let feed = nyaa::get_feed_list(&app.input, &app.category, &app.filter).await;
    app.items.items = feed;
    app.items.select(0);
    sort_feed(app)
}

fn sort_feed(app: &mut App) {
    if let Some(i) = app.sorts.state.selected() {
        if let Some(item) = app.sorts.items.get(i) {
            app.sort = item.to_owned().to_owned();
            app.input_mode = app.last_input_mode.to_owned();
            app.items.items.sort_by(|a, b| match app.sort {
                Sort::Date => b.index.cmp(&a.index),
                Sort::Downloads => b.downloads.cmp(&a.downloads),
                Sort::Seeders => b.seeders.cmp(&a.seeders),
                Sort::Leechers => b.leechers.cmp(&a.leechers),
                Sort::Name => b.title.cmp(&a.title),
                Sort::Category => (b.category as u32).cmp(&(a.category as u32))
            });
        }
    }
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    app.categories.select(0);
    app.filters.select(0);
    app.sorts.select(0);
    app.errors.add(r#"Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum."#.to_owned());
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
                    }
                    KeyCode::Char('f') => {
                        app.last_input_mode = app.input_mode.to_owned();
                        app.input_mode = InputMode::SelectFilter;
                    }
                    KeyCode::Char('s') => {
                        app.last_input_mode = app.input_mode.to_owned();
                        app.input_mode = InputMode::SelectSort;
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
                    KeyCode::Char('J') => app.categories.next(4),
                    KeyCode::Char('K') => app.categories.previous(4),
                    KeyCode::Char('g') => app.categories.select(0),
                    KeyCode::Char('G') => app.categories.select(app.categories.items.len() - 1),
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
                    KeyCode::Char('J') => app.filters.next(4),
                    KeyCode::Char('K') => app.filters.previous(4),
                    KeyCode::Char('g') => app.filters.select(0),
                    KeyCode::Char('G') => app.filters.select(app.filters.items.len() - 1),
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
                InputMode::SelectSort => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        app.input_mode = app.last_input_mode.to_owned()
                    }
                    KeyCode::Char('/') => app.input_mode = InputMode::Editing,
                    KeyCode::Char('j') | KeyCode::Down => app.sorts.next(1),
                    KeyCode::Char('k') | KeyCode::Up => app.sorts.previous(1),
                    KeyCode::Char('J') => app.sorts.next(4),
                    KeyCode::Char('K') => app.sorts.previous(4),
                    KeyCode::Char('g') => {
                        app.sorts.select(0);
                    }
                    KeyCode::Char('G') => {
                        app.sorts.select(app.sorts.items.len() - 1);
                    }
                    KeyCode::Enter | KeyCode::Char('l') => {
                        sort_feed(&mut app);
                    }
                    _ => {}
                }
                InputMode::ShowError => match key.code {
                    _ => {
                        app.input_mode = app.last_input_mode.to_owned();
                        let _ = app.errors.remove();
                    }
                }
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
