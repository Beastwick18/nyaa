use std::cmp::{self, Ordering};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Margin, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Cell, Clear, Paragraph, Row, Scrollbar, ScrollbarOrientation, Table,
    },
    Frame,
};

use crate::{
    app::{App, Mode},
    nyaa::{self, Item},
};

use super::{sort::Sort, StatefulTable};

pub struct ResultsWidget {
    table: StatefulTable<nyaa::Item>,
}

impl ResultsWidget {
    pub fn with_items(&mut self, items: Vec<nyaa::Item>, sort: &Sort) {
        let len = items.len();
        self.table.items = items;
        self.sort(sort);
        self.table.select(0);
        self.table.scrollbar_state = self.table.scrollbar_state.content_length(len);
    }

    pub fn sort(&mut self, sort: &Sort) {
        let f: fn(&Item, &Item) -> Ordering = match sort {
            Sort::Date => |a, b| a.index.cmp(&b.index),
            Sort::Downloads => |a, b| b.downloads.cmp(&a.downloads),
            Sort::Seeders => |a, b| b.seeders.cmp(&a.seeders),
            Sort::Leechers => |a, b| b.leechers.cmp(&a.leechers),
            Sort::Name => |a, b| b.title.cmp(&a.title),
            Sort::Category => |a, b| b.category.cmp(&a.category),
        };
        self.table.items.sort_by(f);
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

fn shorten_number(mut n: u32) -> String {
    if n >= 10000 {
        n /= 1000;
        return n.to_string() + "K";
    }
    n.to_string()
}

impl super::Widget for ResultsWidget {
    fn draw(&self, f: &mut Frame, app: &App, area: Rect) {
        let focus_color = match app.mode {
            Mode::Normal => app.theme.border_focused_color,
            _ => app.theme.border_color,
        };
        let binding = [
            Constraint::Length(3),
            Constraint::Length(area.width - 32 as u16),
            Constraint::Length(9),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(5),
        ];
        let header: &[&String] = &[
            &"Cat".to_owned(),
            &"Name".to_owned(),
            &format!("{:^9}", " Size"),
            &format!("{:^4}", ""),
            &format!("{:^4}", ""),
            &format!("{:^4}", ""),
        ];
        let header_cells = header.iter().map(|h| {
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
                        app.theme.trusted
                    } else if item.remake {
                        app.theme.remake
                    } else {
                        app.theme.fg
                    }),
                ),
                Text::raw(format!("{:>9}", item.size.to_string())),
                Text::styled(
                    format!("{:>4}", item.seeders.to_string()),
                    Style::new().fg(app.theme.trusted),
                ),
                Text::styled(
                    format!("{:>4}", item.leechers.to_string()),
                    Style::new().fg(app.theme.remake),
                ),
                Text::raw(format!("{:<5}", shorten_number(item.downloads))),
            ])
            .fg(app.theme.fg)
            .height(1)
            .bottom_margin(0)
        });

        let sb = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .track_symbol(Some("│"))
            .begin_symbol(Some(""))
            .end_symbol(None);
        let sb_area = area.inner(&Margin {
            vertical: 1,
            horizontal: 0,
        });

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
        f.render_stateful_widget(sb, sb_area, &mut self.table.scrollbar_state.to_owned());
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
                KeyCode::Char('J') => {
                    self.table.next(4);
                }
                KeyCode::Char('K') => {
                    self.table.next(-4);
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
