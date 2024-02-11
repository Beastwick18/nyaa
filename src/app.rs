use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{backend::Backend, Terminal};

use crate::{
    ui,
    widget::{category::CategoryPopup, search::SearchWidget, sort::SortPopup, Popup, Widget},
};

pub enum Mode {
    Normal,
    Search,
    Category,
    Sort,
}

pub struct App {
    pub mode: Mode,
}

pub struct Widgets {
    pub category: CategoryPopup,
    pub sort: SortPopup,
    pub search: SearchWidget,
    // TODO: Add query struct containing category, filter, etc. updated by popups
}

impl Default for App {
    fn default() -> Self {
        App { mode: Mode::Normal }
    }
}

impl Default for Widgets {
    fn default() -> Self {
        Widgets {
            category: CategoryPopup::default(),
            sort: SortPopup::default(),
            search: SearchWidget::default(),
        }
    }
}

fn normal_event(app: &mut App, e: &Event) {
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
            KeyCode::Char('/') => {
                app.mode = Mode::Search;
            }
            _ => {}
        }
    }
}

pub async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    let mut w = Widgets::default();
    loop {
        terminal.draw(|f| ui::draw(&w, &app, f))?;

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
            }
            Mode::Search => {
                w.search.handle_event(&mut app, &evt);
            }
        }

        // Global key bindings that will always be checked, regardless of the current mode
        if let Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            ..
        }) = evt
        {
            match code {
                KeyCode::Char('q') => return Ok(()),
                _ => {}
            }
        }
    }
}
