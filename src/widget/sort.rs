use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::Constraint,
    widgets::{Row, Table},
};

use crate::{app::Mode, ui};

use super::{EnumIter, Popup};

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
    pub sort: Sort,
}

impl Default for SortPopup {
    fn default() -> Self {
        return SortPopup { sort: Sort::Date };
    }
}

impl Popup for SortPopup {
    fn draw(&self, f: &mut ratatui::prelude::Frame) {
        let area = super::centered_rect(30, 11 as u16 + 2, f.size());
        let table = Table::new(
            Sort::iter().map(|item| Row::new(vec![item.to_string()])),
            &[Constraint::Percentage(100)],
        )
        .block(ui::HI_BLOCK.clone());
        f.render_widget(table, area);
    }

    fn handle_event(&mut self, app: &mut crate::app::App, e: &crossterm::event::Event) {
        if let Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            ..
        }) = e
        {
            match code {
                KeyCode::Esc | KeyCode::Char('s') => {
                    app.mode = Mode::Normal;
                }
                _ => {}
            }
        }
    }
}
