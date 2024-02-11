use crate::config::Config;
use crate::{
    app::{App, InputMode},
    nyaa::{Item, Sort},
};
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use queues::IsQueue;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::cmp::Ordering;
use std::{
    io,
    process::{Command, Stdio},
};

mod app;
mod config;
mod nyaa;
mod ui;

async fn search_nyaa(app: &mut App) {
    app.input_mode = InputMode::Normal;
    app.table.items.clear();

    match nyaa::get_feed_list(&app.input, &app.category.selected, &app.filter.selected).await {
        Ok(feed) => {
            app.table.items = feed;
            app.table.select(0);
        }
        Err(e) => {
            let _ = app.errors.add(e.to_string());
        }
    };
    sort_feed(app)
}

fn sort_feed(app: &mut App) {
    if let Some(i) = app.sort.table.state.selected() {
        if let Some(item) = app.sort.table.items.get(i) {
            app.sort.selected = item.to_owned();
            let f: fn(&Item, &Item) -> Ordering = match app.sort.selected {
                Sort::Date => |a, b| a.index.cmp(&b.index),
                Sort::Downloads => |a, b| b.downloads.cmp(&a.downloads),
                Sort::Seeders => |a, b| b.seeders.cmp(&a.seeders),
                Sort::Leechers => |a, b| b.leechers.cmp(&a.leechers),
                Sort::Name => |a, b| b.title.cmp(&a.title),
                Sort::Category => |a, b| (b.category as u32).cmp(&(a.category as u32)),
            };
            app.table.items.sort_by(f);
        }
    }
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    app.category.table.select(0);
    app.filter.table.select(0);
    app.sort.table.select(0);
    loop {
        terminal.draw(|f| ui::ui::<B>(f, &mut app))?;
        match app.input_mode {
            InputMode::Searching => {
                search_nyaa(&mut app).await;
                app.last_input_mode = app.input_mode;
                app.input_mode = InputMode::Normal;
                continue; // Skip reading input so it does not block this thread
            }
            _ => {}
        }

        if let Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            ..
        }) = event::read()?
        {
            match app.input_mode {
                InputMode::Normal => match code {
                    KeyCode::F(1) => {
                        app.last_input_mode = app.input_mode;
                        app.input_mode = InputMode::ShowHelp;
                    }
                    KeyCode::Char('h') => {}
                    KeyCode::Char('j') | KeyCode::Down => app.table.next_wrap(1),
                    KeyCode::Char('k') | KeyCode::Up => app.table.next_wrap(-1),
                    KeyCode::Char('J') | KeyCode::Char('}') => app.table.next(4),
                    KeyCode::Char('K') | KeyCode::Char('{') => app.table.next(-4),
                    KeyCode::Char('g') => {
                        app.table.select(0);
                    }
                    KeyCode::Char('G') => {
                        app.table.select(app.table.items.len() - 1);
                    }
                    KeyCode::Char('l') | KeyCode::Enter => {
                        if let Some(i) = app.table.state.selected() {
                            if let Some(item) = app.table.items.get(i) {
                                let cmd_str = app
                                    .config
                                    .torrent_client_cmd
                                    .clone()
                                    .replace("{magnet}", &item.magnet_link)
                                    .replace("{torrent}", &item.torrent_link)
                                    .replace("{title}", &item.title)
                                    .replace("{file}", &item.file_name);

                                if let Ok(cmd) = shellwords::split(&cmd_str) {
                                    if let [exec, first, other @ ..] = cmd.as_slice() {
                                        let args = [&[first.to_owned()], other].concat();
                                        let _ = Command::new(exec)
                                            .args(args)
                                            .stdin(Stdio::null())
                                            .stdout(Stdio::null())
                                            .stderr(Stdio::null())
                                            .spawn();
                                    } else if let Some(p) = Config::get_path().ok() {
                                        let _ = app.errors.add(format!("The command found in {}:\n\n\"{}\"\n\n...is not valid.",
                                                                           p.to_str().unwrap_or_default().to_owned(),
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
                InputMode::Editing => match code {
                    KeyCode::F(1) => {
                        app.last_input_mode = app.input_mode.to_owned();
                        app.input_mode = InputMode::ShowHelp;
                    }
                    KeyCode::Enter => {
                        app.table.items.clear();
                        app.input_mode = InputMode::Loading;
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
                        code,
                        |_, _| {
                            should_refresh = true;
                        },
                    ) {
                        app.input_mode = mode;
                    }
                    if should_refresh {
                        app.table.items.clear();
                        app.input_mode = InputMode::Loading;
                    }
                }
                InputMode::SelectFilter => {
                    let mut should_refresh = false;
                    if let Some(mode) =
                        app.filter
                            .handle_keybinds(app.last_input_mode.to_owned(), code, |_, _| {
                                should_refresh = true;
                            })
                    {
                        app.input_mode = mode;
                    }
                    if should_refresh {
                        app.table.items.clear();
                        app.input_mode = InputMode::Loading;
                    }
                }
                InputMode::SelectSort => {
                    let mut should_sort = false;
                    if let Some(mode) =
                        app.sort
                            .handle_keybinds(app.last_input_mode.to_owned(), code, |_, _| {
                                app.input_mode = app.last_input_mode.to_owned();
                                should_sort = true;
                            })
                    {
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
                InputMode::Loading | InputMode::Searching => {}
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
    match config::Config::from_file() {
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
