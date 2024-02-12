use std::cmp;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::Text,
    widgets::{Cell, Row, Table},
    Frame,
};

use crate::{
    app::{App, Mode},
    ui,
};

use super::StatefulTable;

pub struct ResultsWidget {
    table: StatefulTable<Vec<String>>,
}

impl Default for ResultsWidget {
    fn default() -> Self {
        ResultsWidget {
            table: StatefulTable::with_items(vec![
                vec![
                    "Cat".to_owned(),
                    "Title 1".to_owned(),
                    "1".to_owned(),
                    "2".to_owned(),
                    "3".to_owned(),
                ],
                vec![
                    "Cat".to_owned(),
                    "Title 2".to_owned(),
                    "1".to_owned(),
                    "2".to_owned(),
                    "3".to_owned(),
                ],
                vec![
                    "Cat".to_owned(),
                    "Title 3".to_owned(),
                    "1".to_owned(),
                    "2".to_owned(),
                    "3".to_owned(),
                ],
            ]),
        }
    }
}

impl super::Widget for ResultsWidget {
    fn draw(&self, f: &mut Frame, app: &App, area: Rect) {
        let binding = [
            Constraint::Length(3),
            Constraint::Length(area.width - 21),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(5),
        ];
        static HEADER_CELLS: [&str; 5] = ["Cat", "Name", "", "", "󰇚"];
        let header_cells = HEADER_CELLS.iter().map(|h| {
            Cell::from(Text::raw(*h)).style(Style::default().add_modifier(Modifier::BOLD))
        });
        let header = Row::new(header_cells)
            .style(
                Style::default()
                    .add_modifier(Modifier::UNDERLINED)
                    .fg(Color::White),
            )
            .height(1)
            .bottom_margin(0);

        // let items = app.table.items.iter().map(|item| {
        let items = self
            .table
            .items
            .iter()
            .map(|item| Row::new(item.to_owned()).height(1).bottom_margin(0));

        let table = Table::new(items, [Constraint::Percentage(100)])
            .header(header)
            .block(match app.mode {
                Mode::Normal => ui::HI_BLOCK.to_owned(),
                _ => ui::DEFAULT_BLOCK.to_owned(),
            })
            .highlight_style(Style::default().bg(Color::Rgb(60, 60, 60)))
            .widths(&binding);
        f.render_stateful_widget(table, area, &mut self.table.state.to_owned());
    }

    fn handle_event(&mut self, _app: &mut crate::app::App, e: &crossterm::event::Event) {
        if let Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            ..
        }) = e
        {
            match code {
                KeyCode::Char('j') | KeyCode::Down => {
                    self.table.next(1);
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.table.next(-1);
                }
                KeyCode::Char('G') => {
                    self.table.select(cmp::max(self.table.items.len(), 1) - 1);
                }
                KeyCode::Char('g') => {
                    self.table.select(0);
                }
                _ => {}
            }
        }
    }
}
