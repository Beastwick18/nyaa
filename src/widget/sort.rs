use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Rect},
    style::{Style, Stylize},
    widgets::{Block, Borders, Clear, Row, Table},
    Frame,
};

use crate::app::{App, Mode};

use super::{EnumIter, StatefulTable, Widget};

#[derive(Clone)]
pub enum Sort {
    Date,
    Downloads,
    Seeders,
    Leechers,
    Name,
    Category,
}

impl EnumIter<Sort> for Sort {
    fn iter() -> std::slice::Iter<'static, Sort> {
        static SORTS: &'static [Sort] = &[
            Sort::Date,
            Sort::Downloads,
            Sort::Seeders,
            Sort::Leechers,
            Sort::Name,
            Sort::Category,
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
        let center = super::centered_rect(30, 8, area);
        let clear = super::centered_rect(center.width + 2, center.height, area);
        let items = self.table.items.iter().enumerate().map(|(i, item)| {
            match i == (self.selected.to_owned() as usize) {
                true => Row::new(vec![format!("  {}", item.to_owned())]),
                false => Row::new(vec![format!("   {}", item.to_owned())]),
            }
        });
        let table = Table::new(items, [Constraint::Percentage(100)])
            .block(
                Block::new()
                    .border_style(Style::new().fg(app.theme.border_focused_color))
                    .borders(Borders::ALL)
                    .border_type(app.theme.border)
                    .title("Sort"),
            )
            .fg(app.theme.fg)
            .bg(app.theme.bg)
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
}
