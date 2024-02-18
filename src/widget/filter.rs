use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Rect},
    style::{Style, Stylize},
    widgets::{Block, Clear, Row, Table},
    Frame,
};
use serde::{Deserialize, Serialize};

use crate::app::{App, Mode};

use super::{create_block, EnumIter, StatefulTable, Widget};

#[derive(Clone, Serialize, Deserialize)]
pub enum Filter {
    NoFilter = 0,
    NoRemakes = 1,
    TrustedOnly = 2,
    Batches = 3,
}

impl EnumIter<Filter> for Filter {
    fn iter() -> std::slice::Iter<'static, Filter> {
        static FILTERS: &'static [Filter] = &[
            Filter::NoFilter,
            Filter::NoRemakes,
            Filter::TrustedOnly,
            Filter::Batches,
        ];
        FILTERS.iter()
    }
}

impl ToString for Filter {
    fn to_string(&self) -> String {
        match self {
            Filter::NoFilter => "No Filter".to_owned(),
            Filter::NoRemakes => "No Remakes".to_owned(),
            Filter::TrustedOnly => "Trusted Only".to_owned(),
            Filter::Batches => "Batches".to_owned(),
        }
    }
}

pub struct FilterPopup {
    pub table: StatefulTable<String>,
    pub selected: Filter,
}

impl Default for FilterPopup {
    fn default() -> Self {
        FilterPopup {
            table: StatefulTable::with_items(Filter::iter().map(|item| item.to_string()).collect()),
            selected: Filter::NoFilter,
        }
    }
}

impl Widget for FilterPopup {
    fn draw(&self, f: &mut Frame, app: &App, area: Rect) {
        let center = super::centered_rect(30, 6, area);
        let clear = super::centered_rect(center.width + 2, center.height, area);
        let items = self.table.items.iter().enumerate().map(|(i, item)| {
            match i == (self.selected.to_owned() as usize) {
                true => Row::new(vec![format!("  {}", item.to_owned())]),
                false => Row::new(vec![format!("   {}", item.to_owned())]),
            }
        });
        let table = Table::new(items, [Constraint::Percentage(100)])
            .block(create_block(app.theme, true).title("Filter"))
            .highlight_style(Style::default().bg(app.theme.hl_bg));
        f.render_widget(Clear, clear);
        f.render_widget(Block::new().bg(app.theme.bg), clear);
        f.render_stateful_widget(table, center, &mut self.table.state.to_owned());
    }

    fn handle_event(&mut self, app: &mut crate::app::App, e: &crossterm::event::Event) {
        if let Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            ..
        }) = e
        {
            match code {
                KeyCode::Esc | KeyCode::Char('f') | KeyCode::Char('q') => {
                    app.mode = Mode::Normal;
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    self.table.next_wrap(1);
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.table.next_wrap(-1);
                }
                KeyCode::Char('G') => {
                    self.table.select(self.table.items.len() - 1);
                }
                KeyCode::Char('g') => {
                    self.table.select(0);
                }
                KeyCode::Enter => {
                    if let Some(i) =
                        Filter::iter().nth(self.table.state.selected().unwrap_or_default())
                    {
                        self.selected = i.to_owned();
                        app.mode = Mode::Loading;
                    }
                }
                _ => {}
            }
        }
    }

    fn get_help() -> Option<Vec<(&'static str, &'static str)>> {
        Some(vec![
            ("Enter", "Confirm"),
            ("Esc, f, q", "Close"),
            ("g", "Top"),
            ("G", "Bottom"),
            ("j, ↓", "Down"),
            ("k, ↑", "Up"),
        ])
    }
}
