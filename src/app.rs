use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::Stylize as _,
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
        results::ResultsWidget,
        search::SearchWidget,
        sort::SortPopup,
        theme::{Theme, ThemePopup, THEMES},
        Widget,
    },
};

pub static APP_NAME: &str = "nyaa";

pub enum Mode {
    Normal,
    Search,
    Category,
    Sort,
    Filter,
    Theme,
    Loading,
    Error,
}

pub struct App {
    pub mode: Mode,
    pub theme: &'static Theme,
    pub show_hints: bool,
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
}

impl Default for App {
    fn default() -> Self {
        App {
            mode: Mode::Loading,
            theme: widget::theme::THEMES[0],
            show_hints: false,
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
            (Char('h'), &KeyModifiers::NONE) => {
                app.show_hints = !app.show_hints;
            }
            _ => {}
        }
    }
    return false;
}

pub fn draw(widgets: &mut Widgets, app: &mut App, f: &mut Frame) {
    let layout = Layout::new(
        Direction::Vertical,
        &[
            Constraint::Length(app.show_hints as u16),
            Constraint::Length(3),
            Constraint::Min(1),
        ],
    )
    .split(f.size());

    widgets.search.draw(f, app, layout[1]);
    widgets.results.draw(f, app, layout[2]);
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
            widgets.results.draw(f, app, layout[2]);
            f.render_widget(Paragraph::new("Loading..."), area);
        }
        Mode::Error => {
            widgets
                .error
                .with_error(app.errors.pop().unwrap_or_default());
            widgets.error.draw(f, app, f.size());
            // Show error
        }
        Mode::Normal | Mode::Search => {}
    }
    f.render_widget(
        Paragraph::new(app.config.torrent_client_cmd.to_owned())
            .bg(app.theme.bg)
            .fg(app.theme.border_focused_color),
        layout[0],
    );
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
                if let Err(e) = terminal.clear() {
                    app.errors.push(e.to_string());
                }
                continue;
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
            Mode::Loading => {}
        }

        if app.should_quit {
            return Ok(());
        }
        if app.errors.len() > 0 {
            app.mode = Mode::Error;
        }
    }
}
