use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Rect},
    widgets::{Row, StatefulWidget as _, Table},
    Frame,
};
use serde::{Deserialize, Serialize};

use crate::{
    app::{Context, LoadType, Mode},
    popup_enum, style,
};

use super::{border_block, EnumIter, StatefulTable, Widget};

popup_enum! {
    Sort;
    (0, Date, "Date");
    (1, Downloads, "Downloads");
    (2, Seeders, "Seeders");
    (3, Leechers, "Leechers");
    (4, Size, "Size");
}

#[derive(PartialEq, Clone)]
pub enum SortDir {
    Desc,
    Asc,
}

impl Sort {
    pub fn to_url(self) -> String {
        match self {
            Sort::Date => "id".to_owned(),
            Sort::Downloads => "downloads".to_owned(),
            Sort::Seeders => "seeders".to_owned(),
            Sort::Leechers => "leechers".to_owned(),
            Sort::Size => "size".to_owned(),
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
            table: StatefulTable::new(Sort::iter().map(|item| item.to_string()).collect()),
            selected: Sort::Date,
        }
    }
}

impl Widget for SortPopup {
    fn draw(&mut self, f: &mut Frame, app: &Context, area: Rect) {
        let buf = f.buffer_mut();
        let center = super::centered_rect(30, self.table.items.len() as u16 + 2, area);
        let clear = super::centered_rect(center.width + 2, center.height, area);
        let items = self.table.items.iter().enumerate().map(|(i, item)| {
            Row::new([match i == self.selected.to_owned() as usize {
                true => format!("  {}", item),
                false => format!("   {}", item),
            }])
        });
        let table = Table::new(items, [Constraint::Percentage(100)])
            .block(border_block(app.theme, true).title(
                match app.mode == Mode::Sort(SortDir::Asc) {
                    true => "Sort Ascending",
                    false => "Sort Descending",
                },
            ))
            .highlight_style(style!(bg:app.theme.hl_bg));
        super::clear(clear, buf, app.theme.bg);
        table.render(center, buf, &mut self.table.state);
    }

    fn handle_event(&mut self, app: &mut crate::app::Context, e: &crossterm::event::Event) {
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
                    if let Some(i) = Sort::iter().nth(self.table.state.selected().unwrap_or(0)) {
                        self.selected = i.to_owned();
                        app.ascending = app.mode == Mode::Sort(SortDir::Asc);
                        app.mode = Mode::Loading(LoadType::Sorting);
                        app.notify(format!("Sort by \"{}\"", i.to_string()));
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
