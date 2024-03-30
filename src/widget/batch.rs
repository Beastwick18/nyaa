use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Constraint, Rect},
    widgets::{Clear, Row, Table, Widget},
    Frame,
};

use crate::{
    app::{App, LoadType, Mode},
    source::Item,
};

use super::{border_block, StatefulTable};

pub struct BatchWidget {
    table: StatefulTable<Item>,
}

impl BatchWidget {
    pub fn with_items(&mut self, items: Vec<Item>) {
        self.table.with_items(items);
    }
}

impl Default for BatchWidget {
    fn default() -> Self {
        BatchWidget {
            table: StatefulTable::empty(),
        }
    }
}

impl super::Widget for BatchWidget {
    fn draw(&mut self, f: &mut Frame, app: &App, area: Rect) {
        let buf = f.buffer_mut();
        let block = border_block(app.theme, app.mode == Mode::Search).title("Batch");
        let rows = self
            .table
            .items
            .iter()
            .map(|i| Row::new([i.title.to_owned()]))
            .collect::<Vec<Row>>();
        let table = Table::new(rows, [Constraint::Percentage(100)]).block(block);
        Clear.render(area, buf);
        table.render(area, buf)
        // block.render(area, buf);
    }

    fn handle_event(&mut self, app: &mut crate::app::App, evt: &Event) {
        if let Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            modifiers,
            ..
        }) = evt
        {
            use KeyCode::*;
            match (code, modifiers) {
                (Esc, &KeyModifiers::NONE) => {
                    app.mode = Mode::Normal;
                }
                (Enter, &KeyModifiers::NONE) => {
                    app.mode = Mode::Loading(LoadType::Searching);
                    app.page = 1; // Go back to first page
                }
                _ => {}
            };
        }
    }

    fn get_help() -> Option<Vec<(&'static str, &'static str)>> {
        Some(vec![("Enter", "Confirm"), ("Esc", "Stop")])
    }
}
