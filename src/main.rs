use std::env;
use std::process::{Command,Stdio};
use log::{warn, info};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use unicode_width::UnicodeWidthStr;


mod logging;
mod nyaa;

struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
    nyaa_items: Vec<nyaa::Item>
}

impl<T> StatefulList<T> {
    fn with_items(items: Vec<T>, nyaa_items: Vec<nyaa::Item>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
            nyaa_items
        }
    }

    fn next(&mut self, amt: usize) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - amt {
                    self.items.len() - 1
                } else {
                    i + amt
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self, amt: usize) {
        let i = match self.state.selected() {
            Some(i) => {
                if i <= amt - 1 {
                    0
                } else {
                    i - amt
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn unselect(&mut self) {
        self.state.select(None);
    }
}

struct App {
    /// Current value of the input box
    // #[derive(Clone)]
    input: String,
    /// Current input mode
    input_mode: InputMode,
    /// History of recorded messages
    // messages: Vec<String>,
    items: StatefulList<String>
}

enum InputMode {
    Normal,
    Editing,
}

impl Default for App {
    fn default() -> App {
        App {
            input: String::new(),
            input_mode: InputMode::Editing,
            items: StatefulList::with_items(Vec::new(), Vec::new())
        }
    }
}

async fn get_feed_list(query: &String) -> Vec<nyaa::Item> {
    let feed = match nyaa::get_feed(query.to_string()).await {
        Ok(x) => x,
        Err(_) => panic!("Failed to connect to nyaa.si...")
    };
    let mut items: Vec<nyaa::Item> = Vec::new();
    for item in &feed.items {
        if let (Some(ext_map), Some(title), Some(link)) = (item.extensions().get("nyaa"), &item.title, &item.link) {
            let seeders = nyaa::get_ext_value::<u32>(ext_map, "seeders").await.unwrap_or_default();
            let leechers = nyaa::get_ext_value(ext_map, "leechers").await.unwrap_or_default();
            let downloads = nyaa::get_ext_value(ext_map, "downloads").await.unwrap_or_default();
            
            items.push(nyaa::Item {
                seeders,
                leechers,
                downloads,
                title: title.to_string(),
                torrent_link: link.to_string()
            });
        } else {
            warn!("Missing nyaa details");
        }
    }
    return items;
    
    // let mut feed_list: Vec<String> = Vec::new();
    // for item in &items {
    //     feed_list.push(format!(" {:<4} |  {:<4} |  {:<4} | {}", item.downloads, item.seeders, item.leechers, item.title));
    // }
    // return feed_list;
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(f.size());

    let (msg, style) = match app.input_mode {
        InputMode::Normal => (
            vec![
                Span::raw("Press "),
                Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to exit, "),
                Span::styled("/", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to start editing."),
            ],
            Style::default().add_modifier(Modifier::RAPID_BLINK),
        ),
        InputMode::Editing => (
            vec![
                Span::raw("Press "),
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to stop editing, "),
                Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to record the message"),
            ],
            Style::default(),
        ),
    };
    let mut text = Text::from(Spans::from(msg));
    text.patch_style(style);
    let help_message = Paragraph::new(text);
    f.render_widget(help_message, chunks[0]);

    let input = Paragraph::new(app.input.as_ref())
        .style(match app.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Style::default().fg(Color::Yellow),
        })
        .block(Block::default().borders(Borders::ALL).title("Input"));
    f.render_widget(input, chunks[1]);
    match app.input_mode {
        InputMode::Normal =>
            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
            {}

        InputMode::Editing => {
            // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
            f.set_cursor(
                // Put cursor past the end of the input text
                chunks[1].x + app.input.width() as u16 + 1,
                // Move one line down, from the border to the input line
                chunks[1].y + 1,
            )
        }
    }

    let items: Vec<ListItem> = app
        .items
        .items
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let content = vec![Spans::from(Span::raw(m))];
            ListItem::new(content)
        })
        .collect();
    
    /// TODO: Change to table, with name, seed, leech, downloads in seperate columns
    /// maybe abbreviate numbers "15029 -> 15k"
    let items = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("List"))
        .highlight_style(
            Style::default()
                .bg(Color::White)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );
    
    
    f.render_stateful_widget(items, chunks[2], &mut app.items.state);
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match app.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('h') => {},
                    KeyCode::Char('j') => app.items.next(1),
                    KeyCode::Char('k') => app.items.previous(1),
                    KeyCode::Char('J') => app.items.next(4),
                    KeyCode::Char('K') => app.items.previous(4),
                    KeyCode::Char('g') => {
                        app.items.state.select(Some(0));
                    }
                    KeyCode::Char('G') => {
                        app.items.state.select(Some(app.items.items.len() - 1));
                    }
                    KeyCode::Char('l') | KeyCode::Enter => {
                        let i = match app.items.state.selected() {
                            Some(i) => i,
                            None => 0
                        };
                        if let Some(i) = app.items.state.selected() {
                            if let Some(item) = app.items.nyaa_items.get(i) {
                                Command::new("webtorrent-desktop")
                                    .args([item.torrent_link.clone()])
                                    .stdin(Stdio::null())
                                    .stderr(Stdio::null())
                                    .spawn()
                                    .expect("Failed");
                            }
                        }
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
                        // app.items.items.push(app.input.clone());
                        app.input_mode = InputMode::Normal;
                        app.items.items.clear();
                        
                        let feed = get_feed_list(&app.input).await;
                        
                        let mut feed_list: Vec<String> = Vec::new();
                        for item in &feed {
                            feed_list.push(format!(" {:<4} |  {:<4} |  {:<4} | {}", item.downloads, item.seeders, item.leechers, item.title));
                        }
                        app.items.items.extend(feed_list);
                        app.items.nyaa_items = feed;
                        app.items.state.select(Some(0));
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

// #[tokio::main()]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     nyaa::config::create_config();
//     if cfg!(windows) {
//         println!("Windows");
//     } else if cfg!(unix) {
//         println!("Unix");
//     }
    
//     let args: Vec<String> = env::args().collect();
    
//     log::set_logger(&logging::SimpleLogger).unwrap();
//     log::set_max_level(log::LevelFilter::Debug);
    
//     let query = args[1..].to_owned().into_iter().map(|x| x.to_string()).reduce(|x: String, y: String| x + " " + &y).unwrap_or_default();
//     info!("Query: \"{}\"", query);
    
//     let feed = match nyaa::get_feed(query).await {
//         Ok(x) => x,
//         Err(_) => panic!("Failed to connect to nyaa.si...")
//     };
//     let mut items: Vec<nyaa::Item> = Vec::new();
//     for item in &feed.items {
//         if let (Some(ext_map), Some(title), Some(link)) = (item.extensions().get("nyaa"), &item.title, &item.link) {
//             let seeders = nyaa::get_ext_value::<u32>(ext_map, "seeders").await.unwrap_or_default();
//             let leechers = nyaa::get_ext_value(ext_map, "leechers").await.unwrap_or_default();
//             let downloads = nyaa::get_ext_value(ext_map, "downloads").await.unwrap_or_default();
            
//             items.push(nyaa::Item {
//                 seeders,
//                 leechers,
//                 downloads,
//                 title: title.to_string(),
//                 torrent_link: link.to_string()
//             });
//         } else {
//             warn!("Missing nyaa details");
//         }
//     }
    
//     for item in &items {
//         println!(" {:<4} |  {:<4} |  {:<4} | {}", item.downloads, item.seeders, item.leechers, item.title);
//     }
//     let mut buffer = String::new();
//     io::stdin().read_line(&mut buffer).expect("Failed to read from stdin");
//     let n = buffer.trim().parse::<usize>().expect("Failed to convert input to usize");
//     if let Some(x) = items.get(n) {
//         info!("{}: {}", x.title, x.torrent_link);
//     }
    
//     Ok(())
// }
