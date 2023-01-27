use std::process::{Command,Stdio};

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

use crate::app::{InputMode, App};

mod logging;
mod nyaa;
mod app;

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| app::ui(f, &mut app))?;
        
        if let Event::Key(key) = event::read()? {
            match app.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('h') => {},
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
                                let _ = Command::new("/mnt/c/Users/Brad/AppData/Local/WebTorrent/WebTorrent.exe")
                                    .args([item.torrent_link.clone()])
                                    .stdin(Stdio::null())
                                    .stderr(Stdio::null())
                                    .spawn();
                            }
                        }
                    }
                    KeyCode::Char('c') => {
                        todo!("Categories")
                    }
                    KeyCode::Char('f') => {
                        todo!("Filter")
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
                        app.input_mode = InputMode::Normal;
                        app.items.items.clear();
                        
                        let feed = nyaa::get_feed_list(&app.input, &app.category, &app.filter).await;
                        
                        app.items.items = feed;
                        app.items.select(0);
                    }
                    KeyCode::Char(c) => { app.input.push(c); },
                    KeyCode::Backspace => { app.input.pop(); },
                    KeyCode::Esc => { app.input_mode = InputMode::Normal; }
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
