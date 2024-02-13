use std::cmp;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Rect},
    style::{Modifier, Style, Stylize},
    text::Text,
    widgets::{Block, Borders, Cell, Clear, Row, Table},
    Frame,
};

use crate::{
    app::{App, Mode},
    nyaa,
};

use super::StatefulTable;

pub struct ResultsWidget {
    table: StatefulTable<nyaa::Item>,
}

impl ResultsWidget {
    pub fn with_items(&mut self, items: Vec<nyaa::Item>) {
        self.table.items = items;
        self.table.select(0);
    }
    pub fn clear(&mut self) {
        self.table.items = vec![];
    }
}

impl Default for ResultsWidget {
    fn default() -> Self {
        ResultsWidget {
            table: StatefulTable::with_items(vec![]),
        }
    }
}

impl super::Widget for ResultsWidget {
    fn draw(&self, f: &mut Frame, app: &App, area: Rect) {
        let focus_color = match app.mode {
            Mode::Normal => app.theme.border_focused_color,
            _ => app.theme.border_color,
        };
        let binding = [
            Constraint::Length(3),
            Constraint::Length(area.width - 21),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(5),
        ];
        static HEADER_CELLS: &'static [&str] = &["Cat", "Name", "", "", "󰇚"];
        let header_cells = HEADER_CELLS.iter().map(|h| {
            Cell::from(Text::raw(*h)).style(Style::default().add_modifier(Modifier::BOLD))
        });
        let header = Row::new(header_cells)
            .style(
                Style::default()
                    .add_modifier(Modifier::UNDERLINED)
                    .add_modifier(Modifier::BOLD)
                    .fg(focus_color),
            )
            .height(1)
            .bottom_margin(0);

        let items = self.table.items.iter().map(|item| {
            Row::new(vec![
                Text::styled(item.icon.icon, Style::new().fg(item.icon.color)),
                Text::styled(
                    item.title.to_owned(),
                    Style::new().fg(if item.trusted {
                        app.theme.green
                    } else if item.remake {
                        app.theme.red
                    } else {
                        app.theme.fg
                    }),
                ),
                Text::styled(item.seeders.to_string(), Style::new().fg(app.theme.green)),
                Text::styled(item.leechers.to_string(), Style::new().fg(app.theme.red)),
                Text::raw(item.downloads.to_string()),
            ])
            .fg(app.theme.fg)
            .height(1)
            .bottom_margin(0)
        });
        // let items = self.table.items.iter().map(|item| {
        //     Row::new(vec![
        //         item[0].to_owned(),
        //         item[1].to_owned(),
        //         item[2].to_owned(),
        //         item[3].to_owned(),
        //         item[4].to_owned(),
        //     ])
        //     .height(1)
        //     .bottom_margin(0)
        // });

        let table = Table::new(items, [Constraint::Percentage(100)])
            .header(header)
            .block(
                Block::new()
                    .borders(Borders::ALL)
                    .border_type(app.theme.border)
                    .border_style(Style::new().fg(focus_color)),
            )
            .bg(app.theme.bg)
            .highlight_style(Style::default().bg(app.theme.hl_bg))
            .widths(&binding);
        f.render_widget(Clear, area);
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
