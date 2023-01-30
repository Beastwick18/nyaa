use std::{
    process::{Command, Stdio},
    io
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
};
use queues::IsQueue;
use tui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use crate::{
    app::{App, InputMode},
    nyaa::Sort,
};

mod app;
mod logging;
mod nyaa;
mod ui;

async fn search_nyaa(app: &mut App) {
    app.input_mode = InputMode::Normal;
    app.items.items.clear();

    let feed = nyaa::get_feed_list(&app.input, &app.category.selected, &app.filter.selected, app.config.magnet_links).await;
    app.items.items = feed;
    app.items.select(0);
    sort_feed(app)
}

fn sort_feed(app: &mut App) {
    if let Some(i) = app.sort.table.state.selected() {
        if let Some(item) = app.sort.table.items.get(i) {
            app.sort.selected = item.to_owned().to_owned();
            app.items.items.sort_by(|a, b| match app.sort.selected {
                Sort::Date => a.index.cmp(&b.index),
                Sort::Downloads => b.downloads.cmp(&a.downloads),
                Sort::Seeders => b.seeders.cmp(&a.seeders),
                Sort::Leechers => b.leechers.cmp(&a.leechers),
                Sort::Name => b.title.cmp(&a.title),
                Sort::Category => (b.category as u32).cmp(&(a.category as u32)),
            });
        }
    }
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    app.category.table.select(0);
    app.filter.table.select(0);
    app.sort.table.select(0);
    loop {
        terminal.draw(|f| ui::ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match app.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::F(1) => {
                        app.last_input_mode = app.input_mode.to_owned();
                        app.input_mode = InputMode::ShowHelp;
                    }
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
                                let link = item.torrent_link.clone();
                                let cmd_str =
                                    app.config.torrent_client_cmd.clone().replace("%s", &link);

                                if let Ok(cmd) = shellwords::split(&cmd_str) {
                                    if let [exec, first, other @ ..] = cmd.as_slice() {
                                        // app.errors.add(link).unwrap();
                                        let args = [&[first.to_owned()], other].concat();
                                        let _ = Command::new(exec)
                                            .args(args)
                                            .stdin(Stdio::null())
                                            .stdout(Stdio::null())
                                            .stderr(Stdio::null())
                                            .spawn();
                                    } else {
                                        let _ = app.errors.add(format!("The command found in {}:\n\n\"{}\"\n\n...is not valid.",
                                                                       nyaa::config::Config::get_path().to_str().unwrap(),
                                                                       cmd_str));
                                    }
                                }
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
                    KeyCode::F(1) => {
                        app.last_input_mode = app.input_mode.to_owned();
                        app.input_mode = InputMode::ShowHelp;
                    }
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
                InputMode::SelectCategory => {
                    let mut should_refresh = false;
                    if let Some(mode) = app.category.handle_keybinds(
                        app.last_input_mode.to_owned(),
                        key.code,
                        |_, _| {
                            should_refresh = true;
                        },
                    ) {
                        app.input_mode = mode;
                    }
                    if should_refresh {
                        app.input_mode = app.last_input_mode.to_owned();
                        search_nyaa(&mut app).await;
                    }
                }
                InputMode::SelectFilter => {
                    let mut should_refresh = false;
                    if let Some(mode) = app.filter.handle_keybinds(
                        app.last_input_mode.to_owned(),
                        key.code,
                        |_, _| {
                            should_refresh = true;
                        },
                    ) {
                        app.input_mode = mode;
                    }
                    if should_refresh {
                        app.input_mode = app.last_input_mode.to_owned();
                        search_nyaa(&mut app).await;
                    }
                }
                InputMode::SelectSort => {
                    let mut should_sort = false;
                    if let Some(mode) = app.sort.handle_keybinds(
                        app.last_input_mode.to_owned(),
                        key.code,
                        |_, _| {
                            app.input_mode = app.last_input_mode.to_owned();
                            should_sort = true;
                        },
                    ) {
                        app.input_mode = mode;
                    }
                    if should_sort {
                        sort_feed(&mut app);
                    }
                }
                InputMode::ShowError => {
                    app.input_mode = app.last_input_mode.to_owned();
                    let _ = app.errors.remove();
                }
                InputMode::ShowHelp => {
                    app.input_mode = app.last_input_mode.to_owned();
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

    let mut app = App::default();
    match nyaa::config::Config::from_file() {
        Ok(cfg) => {
            app.config = cfg;
        }
        Err(err) => {
            app.errors.add(err.to_string())?;
        }
    };
    app.category.selected = app.config.default_category;
    app.filter.selected = app.config.default_filter;
    app.sort.selected = app.config.default_sort;
    
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
