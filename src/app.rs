use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    widgets::Paragraph,
    Frame, Terminal,
};

use crate::{
    config::Config,
    nyaa,
    widget::{
        self,
        category::{CategoryPopup, ALL_CATEGORIES},
        centered_rect,
        error::ErrorPopup,
        filter::FilterPopup,
        help::HelpPopup,
        results::ResultsWidget,
        search::SearchWidget,
        sort::SortPopup,
        theme::{Theme, ThemePopup, THEMES},
        Widget,
    },
};

pub static APP_NAME: &str = "nyaa";

#[derive(PartialEq, Clone)]
pub enum Mode {
    Normal,
    Search,
    Category,
    Sort,
    Filter,
    Theme,
    Loading,
    Error,
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
            Mode::Loading => "Loading".to_string(),
            Mode::Error => "Error".to_string(),
            Mode::Help => "Help".to_string(),
        }
    }
}

pub struct App {
    pub mode: Mode,
    pub theme: &'static Theme,
    pub should_sort: bool,
    pub config: Config,
    pub errors: Vec<String>,
    should_quit: bool,
}

impl App {
    fn quit(&mut self) {
        self.should_quit = true;
    }
}

pub struct Widgets {
    pub category: CategoryPopup,
    pub sort: SortPopup,
    pub filter: FilterPopup,
    pub theme: ThemePopup,
    pub search: SearchWidget,
    pub results: ResultsWidget,
    pub error: ErrorPopup,
    pub help: HelpPopup,
}

impl Default for App {
    fn default() -> Self {
        App {
            mode: Mode::Loading,
            theme: widget::theme::THEMES[0],
            should_sort: false,
            config: Config::default(),
            errors: vec![],
            should_quit: false,
        }
    }
}

impl Default for Widgets {
    fn default() -> Self {
        Widgets {
            category: CategoryPopup::default(),
            sort: SortPopup::default(),
            filter: FilterPopup::default(),
            theme: ThemePopup::default(),
            search: SearchWidget::default(),
            results: ResultsWidget::default(),
            error: ErrorPopup::default(),
            help: HelpPopup::default(),
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
fn normal_event(app: &mut App, e: &Event) -> bool {
    if let Event::Key(KeyEvent {
        code,
        kind: KeyEventKind::Press,
        modifiers,
        ..
    }) = e
    {
        use KeyCode::*;
        match (code, modifiers) {
            (Char('c'), &KeyModifiers::NONE) => {
                app.mode = Mode::Category;
            }
            (Char('s'), &KeyModifiers::NONE) => {
                app.mode = Mode::Sort;
            }
            (Char('f'), &KeyModifiers::NONE) => {
                app.mode = Mode::Filter;
            }
            (Char('t'), &KeyModifiers::NONE) => {
                app.mode = Mode::Theme;
            }
            (Char('/') | KeyCode::Char('i'), &KeyModifiers::NONE) => {
                app.mode = Mode::Search;
            }
            (Char('q'), &KeyModifiers::NONE) => {
                app.quit();
            }
            _ => {}
        }
    }
    return false;
}

pub fn draw(widgets: &mut Widgets, app: &mut App, f: &mut Frame) {
    let layout = Layout::new(
        Direction::Vertical,
        &[Constraint::Length(3), Constraint::Min(1)],
    )
    .split(f.size());

    widgets.search.draw(f, app, layout[0]);
    widgets.results.draw(f, app, layout[1]);
    match app.mode {
        Mode::Category => {
            widgets.category.draw(f, app, f.size());
        }
        Mode::Sort => {
            widgets.sort.draw(f, app, f.size());
        }
        Mode::Filter => {
            widgets.filter.draw(f, app, f.size());
        }
        Mode::Theme => {
            widgets.theme.draw(f, app, f.size());
        }
        Mode::Loading => {
            let area = centered_rect(10, 1, f.size());
            widgets.results.clear();
            widgets.results.draw(f, app, layout[1]);
            f.render_widget(Paragraph::new("Loading..."), area);
        }
        Mode::Error => {
            widgets
                .error
                .with_error(app.errors.pop().unwrap_or_default());
            widgets.error.draw(f, app, f.size());
        }
        Mode::Help => {
            widgets.help.draw(f, app, f.size());
        }
        Mode::Normal | Mode::Search => {}
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
        Mode::Error => None,
        Mode::Help => None,
        Mode::Loading => None,
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
    w.search.input = app.config.default_search.to_owned();
    w.search.cursor = w.search.input.len();
    w.sort.selected = app.config.default_sort.to_owned();
    w.filter.selected = app.config.default_filter.to_owned();
    for (i, theme) in THEMES.iter().enumerate() {
        if theme.name == app.config.default_theme {
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
        if app.should_sort {
            w.results.sort(&w.sort.selected);
        }
        if app.should_quit {
            return Ok(());
        }
        if app.errors.len() > 0 {
            app.mode = Mode::Error;
        }

        get_help(&mut app, &mut w);
        terminal.draw(|f| draw(&mut w, &mut app, f))?;
        match app.mode {
            Mode::Loading => {
                app.mode = Mode::Normal;
                match nyaa::get_feed_list(
                    &w.search.input,
                    w.category.category,
                    w.filter.selected.to_owned() as usize,
                )
                .await
                {
                    Ok(items) => {
                        w.results.with_items(items, &w.sort.selected);
                    }
                    Err(e) => {
                        app.errors.push(e.to_string());
                    }
                }
                continue; // Redraw
            }
            _ => {}
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
                normal_event(&mut app, &evt);
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
            Mode::Help => {
                w.help.handle_event(&mut app, &evt);
            }
            Mode::Loading => {}
        }
        if app.mode != Mode::Help {
            help_event(&mut app, &evt);
        }
    }
}
