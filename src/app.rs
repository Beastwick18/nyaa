use std::{collections::VecDeque, error::Error};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    Frame, Terminal,
};

use crate::{
    client::Client,
    clip,
    config::Config,
    source::{Item, Sources},
    widget::{
        self,
        batch::BatchWidget,
        category::CategoryPopup,
        clients::ClientsPopup,
        error::ErrorPopup,
        filter::FilterPopup,
        help::HelpPopup,
        page::PagePopup,
        results::ResultsWidget,
        search::SearchWidget,
        sort::{SortDir, SortPopup},
        sources::SourcesPopup,
        theme::{Theme, ThemePopup},
        Widget,
    },
};

pub static APP_NAME: &str = "nyaa";

#[derive(PartialEq, Clone, Copy)]
pub enum LoadType {
    Searching,
    Sorting,
    Filtering,
    Categorizing,
    Downloading,
}

#[derive(PartialEq, Clone)]
pub enum Mode {
    Normal,
    KeyCombo(Vec<char>),
    Search,
    Category,
    Sort(SortDir),
    Filter,
    Theme,
    Sources,
    Clients,
    Loading(LoadType),
    Error,
    Page,
    Help,
}

impl ToString for Mode {
    fn to_string(&self) -> String {
        match self {
            Mode::Normal | Mode::KeyCombo(_) => "Normal".to_string(),
            Mode::Search => "Search".to_string(),
            Mode::Category => "Category".to_string(),
            Mode::Sort(_) => "Sort".to_string(),
            Mode::Filter => "Filter".to_string(),
            Mode::Theme => "Theme".to_string(),
            Mode::Sources => "Sources".to_string(),
            Mode::Clients => "Clients".to_string(),
            Mode::Loading(_) => "Loading".to_string(),
            Mode::Error => "Error".to_string(),
            Mode::Page => "Page".to_owned(),
            Mode::Help => "Help".to_string(),
        }
    }
}

pub struct App {
    pub mode: Mode,
    pub theme: &'static Theme,
    pub config: Config,
    pub errors: VecDeque<String>,
    pub notification: Option<String>,
    pub ascending: bool,
    pub page: usize,
    pub last_page: usize,
    pub total_results: usize,
    pub src: Sources,
    pub client: Client,
    pub batch: Vec<Item>,
    should_quit: bool,
}

// pub struct Context {
//     pub errors: VecDeque<String>,
//     pub theme: &'static Theme,
//     pub config: Config,
//     pub notification: Option<String>,
//     pub sort_ascending: bool,
//     pub page: usize,
//     pub last_page: usize,
//     pub total_results: usize,
// }

impl App {
    pub fn quit(&mut self) {
        self.should_quit = true;
    }
    pub fn show_error<S: ToString>(&mut self, error: S) {
        self.errors.push_back(error.to_string());
    }
    pub fn notify<S: ToString>(&mut self, notification: S) {
        self.notification = Some(notification.to_string());
    }
}

#[derive(Default)]
pub struct Widgets {
    pub batch: BatchWidget,
    pub category: CategoryPopup,
    pub sort: SortPopup,
    pub filter: FilterPopup,
    pub theme: ThemePopup,
    pub sources: SourcesPopup,
    pub clients: ClientsPopup,
    pub search: SearchWidget,
    pub results: ResultsWidget,
    pub error: ErrorPopup,
    pub page: PagePopup,
    pub help: HelpPopup,
}

impl Default for App {
    fn default() -> Self {
        App {
            mode: Mode::Loading(LoadType::Searching),
            theme: widget::theme::THEMES[0],
            config: Config::default(),
            errors: VecDeque::new(),
            notification: None,
            ascending: false,
            page: 1,
            last_page: 1,
            total_results: 0,
            src: Sources::NyaaHtml,
            client: Client::Cmd,
            batch: vec![],
            should_quit: false,
        }
    }
}

fn help_event(app: &mut App, e: &Event) {
    if let Event::Key(KeyEvent {
        code,
        kind: KeyEventKind::Press,
        ..
    }) = e
    {
        match code {
            KeyCode::Char('?') if app.mode != Mode::Search => {
                app.mode = Mode::Help;
            }
            KeyCode::F(1) => {
                app.mode = Mode::Help;
            }
            _ => {}
        }
    }
}

pub fn draw(widgets: &mut Widgets, app: &mut App, f: &mut Frame) {
    let layout_vertical = Layout::new(
        Direction::Vertical,
        [Constraint::Length(3), Constraint::Min(1)],
    )
    .split(f.size());
    let layout_horizontal = Layout::new(
        Direction::Horizontal,
        [Constraint::Ratio(1, 5), Constraint::Ratio(4, 5)],
    )
    .split(layout_vertical[1]);

    widgets.search.draw(f, app, layout_vertical[0]);
    // Dont draw batch pane if none selected
    match app.batch.is_empty() {
        true => widgets.results.draw(f, app, layout_vertical[1]),
        false => {
            widgets.batch.draw(f, app, layout_horizontal[0]);
            widgets.results.draw(f, app, layout_horizontal[1]);
        }
    }
    match app.mode {
        Mode::Category => widgets.category.draw(f, app, f.size()),
        Mode::Sort(_) => widgets.sort.draw(f, app, f.size()),
        Mode::Filter => widgets.filter.draw(f, app, f.size()),
        Mode::Theme => widgets.theme.draw(f, app, f.size()),
        Mode::Error => {
            // Get the oldest error first
            if let Some(error) = app.errors.pop_front() {
                widgets.error.with_error(error);
            }
            widgets.error.draw(f, app, f.size());
        }
        Mode::Help => widgets.help.draw(f, app, f.size()),
        Mode::Page => widgets.page.draw(f, app, f.size()),
        Mode::Sources => widgets.sources.draw(f, app, f.size()),
        Mode::Clients => widgets.clients.draw(f, app, f.size()),
        Mode::KeyCombo(_) | Mode::Normal | Mode::Search | Mode::Loading(_) => {}
    }
}

fn get_help(app: &mut App, w: &mut Widgets) {
    let help = match app.mode {
        Mode::Category => CategoryPopup::get_help(),
        Mode::Sort(_) => SortPopup::get_help(),
        Mode::Normal => ResultsWidget::get_help(),
        Mode::Search => SearchWidget::get_help(),
        Mode::Filter => FilterPopup::get_help(),
        Mode::Theme => ThemePopup::get_help(),
        Mode::Page => PagePopup::get_help(),
        Mode::Sources => SourcesPopup::get_help(),
        Mode::Clients => ClientsPopup::get_help(),
        Mode::Error => None,
        Mode::Help => None,
        Mode::KeyCombo(_) => None,
        Mode::Loading(_) => None,
    };
    if let Some(msg) = help {
        w.help.with_items(msg, app.mode.clone());
        w.help.table.select(0);
    }
}

fn handle_combo(app: &mut App, w: &Widgets, mut keys: Vec<char>, e: &Event) {
    if let Event::Key(KeyEvent {
        code,
        kind: KeyEventKind::Press,
        ..
    }) = e
    {
        match code {
            // Only handle standard chars for now
            KeyCode::Char(c) => keys.push(*c),
            KeyCode::Esc => {
                // Stop combo if esc
                app.mode = Mode::Normal;
                return;
            }
            _ => {}
        }
    }
    match keys[..] {
        ['y', c] => {
            let s = w.results.table.state.selected().unwrap_or(0);
            match w.results.table.items.get(s) {
                Some(item) => {
                    let link = match c {
                        't' => item.torrent_link.to_owned(),
                        'm' => item.magnet_link.to_owned(),
                        'p' => item.post_link.to_owned(),
                        _ => return,
                    };
                    app.mode = Mode::Normal;
                    match clip::copy_to_clipboard(link.to_owned(), app.config.clipboard.clone()) {
                        Ok(_) => app.notify(format!("Copied \"{}\" to clipboard", link)),
                        Err(e) => app.show_error(e),
                    }
                }
                None => {
                    app.show_error("Failed to copy:\nFailed to get torrent link for selected item")
                }
            }
        }
        _ => app.mode = Mode::KeyCombo(keys),
    }
}

pub async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<(), Box<dyn Error>> {
    let w = &mut Widgets::default();
    let config = match Config::load() {
        Ok(config) => config,
        Err(e) => {
            app.show_error(e);
            app.config.clone()
        }
    };
    config.apply(app, w);
    loop {
        if app.should_quit {
            return Ok(());
        }
        if !app.errors.is_empty() {
            app.mode = Mode::Error;
        }

        get_help(app, w);
        w.batch.with_items(app.batch.clone()); // TODO: Find a way to not have to pass this around
        terminal.draw(|f| draw(w, app, f))?;
        if let Mode::Loading(load_type) = app.mode {
            app.mode = Mode::Normal;
            if load_type == LoadType::Downloading {
                let item = match w
                    .results
                    .table
                    .state
                    .selected()
                    .and_then(|i| w.results.table.items.get(i))
                {
                    Some(i) => i,
                    None => continue,
                };
                app.client.clone().download(item, app).await;
                continue;
            }

            let result = app.src.clone().load(load_type, app, w).await;

            match result {
                Ok(items) => w.results.with_items(items, w.sort.selected),
                Err(e) => app.show_error(e),
            }
            continue; // Redraw
        }

        let evt = event::read()?;
        match app.mode.to_owned() {
            Mode::Category => w.category.handle_event(app, &evt),
            Mode::Sort(_) => w.sort.handle_event(app, &evt),
            Mode::Normal => w.results.handle_event(app, &evt),
            Mode::Search => w.search.handle_event(app, &evt),
            Mode::Filter => w.filter.handle_event(app, &evt),
            Mode::Theme => w.theme.handle_event(app, &evt),
            Mode::Error => w.error.handle_event(app, &evt),
            Mode::Page => w.page.handle_event(app, &evt),
            Mode::Help => w.help.handle_event(app, &evt),
            Mode::Sources => w.sources.handle_event(app, &evt),
            Mode::Clients => w.clients.handle_event(app, &evt),
            Mode::KeyCombo(keys) => handle_combo(app, w, keys, &evt),
            Mode::Loading(_) => {}
        }
        if app.mode != Mode::Help {
            help_event(app, &evt);
        }
    }
}
