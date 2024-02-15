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
    nyaa,
    widget::{
        self,
        category::CategoryPopup,
        centered_rect,
        filter::FilterPopup,
        results::ResultsWidget,
        search::SearchWidget,
        sort::SortPopup,
        theme::{Theme, ThemePopup},
        Widget,
    },
};

pub enum Mode {
    Normal,
    Search,
    Category,
    Sort,
    Filter,
    Theme,
    Loading,
}

pub struct App {
    pub mode: Mode,
    pub theme: &'static Theme,
    pub show_hints: bool,
    pub should_sort: bool,
    should_quit: bool,
    // TODO: Add query struct containing category, filter, etc. updated by popups
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
}

impl Default for App {
    fn default() -> Self {
        App {
            mode: Mode::Loading,
            theme: widget::theme::THEMES[0],
            show_hints: false,
            should_sort: false,
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

pub fn draw(widgets: &mut Widgets, app: &App, f: &mut Frame) {
    let layout = Layout::new(
        Direction::Vertical,
        &[
            Constraint::Length(app.show_hints as u16), // TODO: Maybe remove this, keys are obvious. Or make hiding it a config option
            Constraint::Length(3),
            Constraint::Min(1),
        ],
    )
    .split(f.size());

    widgets.search.draw(f, app, layout[1]);
    widgets.results.draw(f, app, layout[2]);
    let mode;
    match app.mode {
        Mode::Normal => {
            mode = "Normal";
        }
        Mode::Category => {
            mode = "Category";
            widgets.category.draw(f, &app, f.size());
        }
        Mode::Sort => {
            mode = "Sort";
            widgets.sort.draw(f, &app, f.size());
        }
        Mode::Search => {
            mode = "Search";
        }
        Mode::Filter => {
            mode = "Filter";
            widgets.filter.draw(f, &app, f.size());
        }
        Mode::Theme => {
            mode = "Theme";
            widgets.theme.draw(f, &app, f.size());
        }
        Mode::Loading => {
            mode = "Loading";
            let area = centered_rect(10, 1, f.size());
            widgets.results.clear();
            widgets.results.draw(f, app, layout[2]);
            f.render_widget(Paragraph::new("Loading..."), area);
        }
    }
    f.render_widget(
        Paragraph::new(mode)
            .bg(app.theme.bg)
            .fg(app.theme.border_focused_color),
        layout[0],
    ); // TODO: Debug only
}

pub async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    let mut w = Widgets::default();
    loop {
        if app.should_sort {
            w.results.sort(&w.sort.selected);
        }
        terminal.draw(|f| draw(&mut w, &app, f))?;
        match app.mode {
            Mode::Loading => {
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
                    Err(_e) => { /* TODO: error */ }
                }
                app.mode = Mode::Normal;
                if let Err(_e) = terminal.clear() {
                    // TODO: handle
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
            Mode::Loading => {}
        }

        if app.should_quit {
            return Ok(());
        }
    }
}
