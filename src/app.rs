use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{backend::Backend, Terminal};

use crate::{
    nyaa, ui,
    widget::{
        self,
        category::CategoryPopup,
        filter::FilterPopup,
        results::ResultsWidget,
        search::SearchWidget,
        sort::SortPopup,
        theme::{Theme, ThemePopup},
        Popup, Widget,
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
        ..
    }) = e
    {
        match code {
            KeyCode::Char('c') => {
                app.mode = Mode::Category;
            }
            KeyCode::Char('s') => {
                app.mode = Mode::Sort;
            }
            KeyCode::Char('f') => {
                app.mode = Mode::Filter;
            }
            KeyCode::Char('t') => {
                app.mode = Mode::Theme;
            }
            KeyCode::Char('/') | KeyCode::Char('i') => {
                app.mode = Mode::Search;
            }
            KeyCode::Char('q') => {
                app.quit();
            }
            _ => {}
        }
    }
    return false;
}

pub async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    let mut w = Widgets::default();
    loop {
        terminal.draw(|f| ui::draw(&mut w, &app, f))?;
        match app.mode {
            Mode::Loading => {
                if let Ok(items) = nyaa::get_feed_list(
                    &(w.search.input).to_owned(),
                    &(w.category.category as u32),
                    &(w.filter.selected.to_owned() as u32),
                )
                .await
                {
                    w.results.with_items(items);
                }
                app.mode = Mode::Normal;
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
