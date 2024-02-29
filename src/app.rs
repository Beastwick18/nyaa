use std::{
    io::{self, BufReader, Read as _},
    process::{Command, Stdio},
};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    Frame, Terminal,
};

use crate::{
    config::Config,
    source::{self, nyaa_html::Item, Sources},
    widget::{
        self,
        category::{CategoryPopup, ALL_CATEGORIES},
        error::ErrorPopup,
        filter::FilterPopup,
        help::HelpPopup,
        page::PagePopup,
        results::ResultsWidget,
        search::SearchWidget,
        sort::SortPopup,
        sources::SourcesPopup,
        theme::{Theme, ThemePopup, THEMES},
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
    Search,
    Category,
    Sort,
    Filter,
    Theme,
    Sources,
    Loading(LoadType),
    Error,
    Page,
    Help,
}

impl ToString for Mode {
    fn to_string(&self) -> String {
        match self {
            Mode::Normal => "Normal".to_string(),
            Mode::Search => "Search".to_string(),
            Mode::Category => "Category".to_string(),
            Mode::Sort => "Sort".to_string(),
            Mode::Filter => "Filter".to_string(),
            Mode::Theme => "Theme".to_string(),
            Mode::Sources => "Sources".to_string(),
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
    pub errors: Vec<String>,
    pub reverse: bool,
    pub page: usize,
    pub last_page: usize,
    pub total_results: usize,
    pub src: Sources,
    should_quit: bool,
}

impl App {
    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}

#[derive(Default)]
pub struct Widgets {
    pub category: CategoryPopup,
    pub sort: SortPopup,
    pub filter: FilterPopup,
    pub theme: ThemePopup,
    pub sources: SourcesPopup,
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
            errors: vec![],
            reverse: false,
            page: 1,
            last_page: 1,
            total_results: 0,
            src: Sources::NyaaHtml,
            should_quit: false,
        }
    }
}

fn download(item: &Item, app: &mut App) {
    #[cfg(target_os = "windows")]
    let cmd_str = app
        .config
        .torrent_client_cmd
        .replace("{magnet}", &item.magnet_link)
        .replace("{torrent}", &item.torrent_link)
        .replace("{title}", &item.title)
        .replace("{file}", &item.file_name);
    app.errors.push(item.torrent_link.clone());
    #[cfg(not(target_os = "windows"))]
    let cmd_str = app
        .config
        .torrent_client_cmd
        .replace("{magnet}", &shellwords::escape(&item.magnet_link))
        .replace("{torrent}", &shellwords::escape(&item.torrent_link))
        .replace("{title}", &shellwords::escape(&item.title))
        .replace("{file}", &shellwords::escape(&item.file_name));
    let cmd = match shellwords::split(&cmd_str) {
        Ok(cmd) => cmd,
        Err(e) => {
            app.errors.push(format!(
                "{}\n{}:\nfailed to split command:\n{}",
                cmd_str, app.config.torrent_client_cmd, e
            ));
            return;
        }
    };
    if let [exec, args @ ..] = cmd.as_slice() {
        let cmd = Command::new(exec)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn();
        let child = match cmd {
            Ok(child) => child,
            Err(e) => {
                app.errors
                    .push(format!("{}:\nFailed to run:\n{}", cmd_str, e));
                return;
            }
        };
        let output = match child.wait_with_output() {
            Ok(output) => output,
            Err(e) => {
                app.errors
                    .push(format!("{}:\nFailed to get output:\n{}", cmd_str, e));
                return;
            }
        };

        if output.status.code() != Some(0) {
            let mut err = BufReader::new(&*output.stderr);
            let mut err_str = String::new();
            err.read_to_string(&mut err_str).unwrap_or(0);
            app.errors.push(format!(
                "{}:\nExited with status code {}:\n{}",
                cmd_str, output.status, err_str
            ));
        }
    } else {
        app.errors
            .push(format!("{}:\nThe command is not valid.", cmd_str));
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
    let layout = Layout::new(
        Direction::Vertical,
        [Constraint::Length(3), Constraint::Min(1)],
    )
    .split(f.size());

    widgets.search.draw(f, app, layout[0]);
    widgets.results.draw(f, app, layout[1]);
    match app.mode {
        Mode::Category => widgets.category.draw(f, app, f.size()),
        Mode::Sort => widgets.sort.draw(f, app, f.size()),
        Mode::Filter => widgets.filter.draw(f, app, f.size()),
        Mode::Theme => widgets.theme.draw(f, app, f.size()),
        Mode::Error => {
            if let Some(error) = app.errors.pop() {
                widgets.error.with_error(error);
            }
            widgets.error.draw(f, app, f.size());
        }
        Mode::Help => widgets.help.draw(f, app, f.size()),
        Mode::Page => widgets.page.draw(f, app, f.size()),
        Mode::Sources => widgets.sources.draw(f, app, f.size()),
        Mode::Normal | Mode::Search | Mode::Loading(_) => {}
    }
}

fn get_help(app: &mut App, w: &mut Widgets) {
    let help = match app.mode {
        Mode::Category => CategoryPopup::get_help(),
        Mode::Sort => SortPopup::get_help(),
        Mode::Normal => ResultsWidget::get_help(),
        Mode::Search => SearchWidget::get_help(),
        Mode::Filter => FilterPopup::get_help(),
        Mode::Theme => ThemePopup::get_help(),
        Mode::Page => PagePopup::get_help(),
        Mode::Sources => SourcesPopup::get_help(),
        Mode::Error => None,
        Mode::Help => None,
        Mode::Loading(_) => None,
    };
    if let Some(msg) = help {
        w.help.with_items(msg, app.mode.clone());
        w.help.table.select(0);
    }
}

pub async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    let mut w = Widgets::default();
    app.config = match Config::from_file() {
        Ok(config) => config,
        Err(e) => {
            app.errors.push(e.to_string());
            app.config
        }
    };
    w.search.input.input = app.config.default_search.to_owned();
    w.search.input.cursor = w.search.input.input.len();
    w.sort.selected = app.config.default_sort.to_owned();
    w.filter.selected = app.config.default_filter.to_owned();
    app.src = app.config.default_source.to_owned();
    let theme_name = app.config.default_theme.to_lowercase();
    for (i, theme) in THEMES.iter().enumerate() {
        if theme.name.to_lowercase() == theme_name {
            w.theme.selected = i;
            app.theme = theme;
            break;
        }
    }
    for cat in ALL_CATEGORIES {
        if let Some(ent) = cat
            .entries
            .iter()
            .find(|ent| ent.cfg == app.config.default_category)
        {
            w.category.category = ent.id;
            break;
        }
    }
    loop {
        if app.should_quit {
            return Ok(());
        }
        if !app.errors.is_empty() {
            app.mode = Mode::Error;
        }

        get_help(&mut app, &mut w);
        terminal.draw(|f| draw(&mut w, &mut app, f))?;
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
                download(item, &mut app);
                continue;
            }

            let result = source::load(app.src, load_type, &mut app, &w).await;

            match result {
                Ok(items) => {
                    w.results.with_items(items);
                }
                Err(e) => {
                    app.errors.push(e.to_string());
                }
            }
            continue; // Redraw
        }

        let evt = event::read()?;
        match app.mode {
            Mode::Category => {
                w.category.handle_event(&mut app, &evt);
            }
            Mode::Sort => {
                w.sort.handle_event(&mut app, &evt);
            }
            Mode::Normal => {
                w.results.handle_event(&mut app, &evt);
            }
            Mode::Search => {
                w.search.handle_event(&mut app, &evt);
            }
            Mode::Filter => {
                w.filter.handle_event(&mut app, &evt);
            }
            Mode::Theme => {
                w.theme.handle_event(&mut app, &evt);
            }
            Mode::Error => {
                w.error.handle_event(&mut app, &evt);
            }
            Mode::Page => {
                w.page.handle_event(&mut app, &evt);
            }
            Mode::Help => {
                w.help.handle_event(&mut app, &evt);
            }
            Mode::Sources => {
                w.sources.handle_event(&mut app, &evt);
            }
            Mode::Loading(_) => {}
        }
        if app.mode != Mode::Help {
            help_event(&mut app, &evt);
        }
    }
}
