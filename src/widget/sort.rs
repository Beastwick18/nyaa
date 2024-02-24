use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Rect},
    style::Style,
    widgets::{Row, Table},
    Frame,
};
use serde::{Deserialize, Serialize};

use crate::app::{App, Mode};

use super::{create_block, EnumIter, StatefulTable, Widget};

#[derive(Clone, Serialize, Deserialize)]
pub enum Sort {
    Date,
    Downloads,
    Seeders,
    Leechers,
    Name,
    Category,
    Size,
}

impl EnumIter<Sort> for Sort {
    fn iter() -> std::slice::Iter<'static, Sort> {
        static SORTS: &[Sort] = &[
            Sort::Date,
            Sort::Downloads,
            Sort::Seeders,
            Sort::Leechers,
            Sort::Name,
            Sort::Category,
            Sort::Size,
        ];
        SORTS.iter()
    }
}

impl ToString for Sort {
    fn to_string(&self) -> String {
        match self {
            Sort::Date => "Date".to_owned(),
            Sort::Downloads => "Downloads".to_owned(),
            Sort::Seeders => "Seeders".to_owned(),
            Sort::Leechers => "Leechers".to_owned(),
            Sort::Name => "Name".to_owned(),
            Sort::Category => "Category".to_owned(),
            Sort::Size => "Size".to_owned(),
        }
    }
}

pub struct SortPopup {
    pub table: StatefulTable<String>,
    pub selected: Sort,
}

impl Default for SortPopup {
    fn default() -> Self {
        SortPopup {
            table: StatefulTable::with_items(Sort::iter().map(|item| item.to_string()).collect()),
            selected: Sort::Date,
        }
    }
}

impl Widget for SortPopup {
    fn draw(&self, f: &mut Frame, app: &App, area: Rect) {
        let center = super::centered_rect(30, 9, area);
        let clear = super::centered_rect(center.width + 2, center.height, area);
        let items = self.table.items.iter().enumerate().map(|(i, item)| {
            Row::new(vec![match i == self.selected.to_owned() as usize {
                true => format!("  {}", item.to_owned()),
                false => format!("   {}", item.to_owned()),
            }])
        });
        let table = Table::new(items, [Constraint::Percentage(100)])
            .block(create_block(app.theme, true).title("Sort"))
            .highlight_style(Style::default().bg(app.theme.hl_bg));
        super::clear(f, clear, app.theme.bg);
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
                KeyCode::Esc | KeyCode::Char('s') | KeyCode::Char('q') => {
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
                        Sort::iter().nth(self.table.state.selected().unwrap_or_default())
                    {
                        self.selected = i.to_owned();
                        app.mode = Mode::Normal;
                        app.should_sort = true;
                    }
                }
                _ => {}
            }
        }
    }

    fn get_help() -> Option<Vec<(&'static str, &'static str)>> {
        Some(vec![
            ("Enter", "Confirm"),
            ("Esc, s, q", "Close"),
            ("j, ↓", "Down"),
            ("k, ↑", "Up"),
            ("g", "Top"),
            ("G", "Bottom"),
        ])
    }
}
