use std::cmp::max;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Constraint, Margin, Rect},
    style::{Modifier, Style, Stylize},
    text::Text,
    widgets::{
        Clear, Paragraph, Row, Scrollbar, ScrollbarOrientation, StatefulWidget, Table, Widget,
    },
    Frame,
};

use crate::{
    app::{App, LoadType, Mode},
    source::Item,
    widget::sort::SortDir,
};

use super::{border_block, centered_rect, sort::Sort, StatefulTable};

pub struct ResultsWidget {
    pub table: StatefulTable<Item>,
    sort: Sort,
}

impl ResultsWidget {
    pub fn with_items(&mut self, items: Vec<Item>, sort: Sort) {
        let len = items.len();
        self.table.items = items;
        self.table.select(0);
        self.table.scrollbar_state = self.table.scrollbar_state.content_length(len);
        self.sort = sort;
    }
}

impl Default for ResultsWidget {
    fn default() -> Self {
        ResultsWidget {
            table: StatefulTable::with_items(vec![]),
            sort: Sort::Date,
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
        let size = f.size();
        let buf = f.buffer_mut();
        let focus_color = match app.mode {
            Mode::Normal => app.theme.border_focused_color,
            _ => app.theme.border_color,
        };
        let raw_date_width = self.table.items.first().map(|i| i.date.len()).unwrap_or(10) as u16;
        let date_width = max(raw_date_width, 6);
        let title_width = max(area.width as i32 - 32 - date_width as i32, 5) as u16;
        let binding = Constraint::from_lengths([3, title_width, 9, date_width, 4, 4, 5]);
        let header_slice = &mut [
            "Cat".to_owned(),
            "Name".to_owned(),
            format!("  {}", "Size"),
            format!(
                "{:^width$}",
                "Date  ",
                width = max(raw_date_width, 4) as usize + 2
            ),
            format!(" {}", ""),
            format!(" {}", ""),
            format!(" {}", ""),
        ];
        let direction = match app.ascending {
            true => "▲",
            false => "▼",
        };
        let sort_idx = match self.sort {
            Sort::Date => 3,
            Sort::Size => 2,
            Sort::Seeders => 4,
            Sort::Leechers => 5,
            Sort::Downloads => 6,
        };
        let sort_text = format!("{} {}", header_slice[sort_idx].trim(), direction);
        let sort_fmt = match self.sort {
            Sort::Size => format!("  {:<8}", sort_text),
            Sort::Date => format!(
                "{:^width$}",
                sort_text,
                width = max(raw_date_width, 4) as usize + 2
            ),
            Sort::Seeders => format!(" {:<3}", sort_text),
            Sort::Leechers => format!(" {:<3}", sort_text),
            Sort::Downloads => format!(" {:<3}", sort_text),
        };
        header_slice[sort_idx] = sort_fmt;
        let header = Row::new(header_slice.to_owned())
            .add_modifier(Modifier::BOLD)
            .add_modifier(Modifier::UNDERLINED)
            .fg(focus_color)
            .height(1)
            .bottom_margin(0);

        Clear.render(area, buf);
        let items: Vec<Row> = match app.mode {
            Mode::Loading(_) => {
                let area = centered_rect(8, 1, size);
                Paragraph::new("Loading…").render(area, buf);
                vec![]
            }
            _ => self
                .table
                .items
                .iter()
                .map(|item| {
                    Row::new(vec![
                        Text::styled(item.icon.label, Style::new().fg(item.icon.color)),
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
                        Text::raw(format!("{:>9}", item.size)),
                        Text::raw(format!("{:<14}", item.date)),
                        Text::styled(
                            format!("{:>4}", item.seeders),
                            Style::new().fg(app.theme.trusted),
                        ),
                        Text::styled(
                            format!("{:>4}", item.leechers),
                            Style::new().fg(app.theme.remake),
                        ),
                        Text::raw(shorten_number(item.downloads)),
                    ])
                    .fg(app.theme.fg)
                    .height(1)
                    .bottom_margin(0)
                })
                .collect(),
        };
        let sb = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .track_symbol(Some("│"))
            .begin_symbol(Some(""))
            .end_symbol(None);
        let sb_area = area.inner(&Margin {
            vertical: 1,
            horizontal: 0,
        });

        let num_items = items.len();
        let first_item = (app.page - 1) * 75;
        let table = Table::new(items, [Constraint::Percentage(100)])
            .header(header)
            .block(
                border_block(app.theme, app.mode == Mode::Normal).title(format!(
                    "Results {}-{} ({} total): Page {}/{}",
                    first_item + 1,
                    num_items + first_item,
                    app.total_results,
                    app.page,
                    app.last_page
                )),
            )
            .highlight_style(Style::default().bg(app.theme.hl_bg))
            .widths(&binding);
        StatefulWidget::render(table, area, buf, &mut self.table.state.to_owned());
        StatefulWidget::render(sb, sb_area, buf, &mut self.table.scrollbar_state.to_owned());

        let source_str = format!("Source: {}", app.src.to_string());
        let text = Paragraph::new(source_str.clone());
        let right = Rect::new(
            area.right() - 1 - source_str.len() as u16,
            area.top(),
            source_str.len() as u16,
            1,
        );
        f.render_widget(text, right);

        match app.mode {
            Mode::Loading(_) => {}
            _ => {
                if num_items == 0 {
                    let center = centered_rect(10, 1, f.size());
                    f.render_widget(Paragraph::new("No results"), center);
                }
            }
        }
    }

    fn handle_event(&mut self, app: &mut crate::app::App, e: &crossterm::event::Event) {
        if let Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            modifiers,
            ..
        }) = e
        {
            use KeyCode::*;
            match (code, modifiers) {
                (Char('c'), &KeyModifiers::NONE) => {
                    app.mode = Mode::Category;
                }
                (Char('s'), &KeyModifiers::NONE) => {
                    app.mode = Mode::Sort(SortDir::Desc);
                }
                (Char('S'), &KeyModifiers::SHIFT) => {
                    app.mode = Mode::Sort(SortDir::Asc);
                }
                (Char('f'), &KeyModifiers::NONE) => {
                    app.mode = Mode::Filter;
                }
                (Char('t'), &KeyModifiers::NONE) => {
                    app.mode = Mode::Theme;
                }
                (Char('/') | Char('i'), &KeyModifiers::NONE) => {
                    app.mode = Mode::Search;
                }
                (Char('p'), &KeyModifiers::CONTROL) => {
                    app.mode = Mode::Page;
                }
                (Char('p') | Char('h') | Left, &KeyModifiers::NONE) => {
                    if app.page > 1 {
                        app.page -= 1;
                        app.mode = Mode::Loading(LoadType::Searching);
                    }
                }
                (Char('n') | Char('l') | Right, &KeyModifiers::NONE) => {
                    if app.page < app.last_page {
                        app.page += 1;
                        app.mode = Mode::Loading(LoadType::Searching);
                    }
                }
                (Char('r'), &KeyModifiers::NONE) => {
                    app.mode = Mode::Loading(LoadType::Searching);
                }
                (Char('q'), &KeyModifiers::NONE) => {
                    app.quit();
                }
                (Char('j') | KeyCode::Down, &KeyModifiers::NONE) => {
                    self.table.next(1);
                }
                (Char('k') | KeyCode::Up, &KeyModifiers::NONE) => {
                    self.table.next(-1);
                }
                (Char('J'), &KeyModifiers::SHIFT) => {
                    self.table.next(4);
                }
                (Char('K'), &KeyModifiers::SHIFT) => {
                    self.table.next(-4);
                }
                (Char('G'), &KeyModifiers::SHIFT) => {
                    self.table.select(max(self.table.items.len(), 1) - 1);
                }
                (Char('g'), &KeyModifiers::NONE) => {
                    self.table.select(0);
                }
                (Char('H') | Char('P'), &KeyModifiers::SHIFT) => {
                    if app.page != 1 {
                        app.page = 1;
                        app.mode = Mode::Loading(LoadType::Searching);
                    }
                }
                (Char('L') | Char('N'), &KeyModifiers::SHIFT) => {
                    if app.page != app.last_page && app.last_page > 0 {
                        app.page = app.last_page;
                        app.mode = Mode::Loading(LoadType::Searching);
                    }
                }
                (Enter, &KeyModifiers::NONE) => {
                    app.mode = Mode::Loading(LoadType::Downloading);
                }
                (Char('s'), &KeyModifiers::CONTROL) => {
                    app.mode = Mode::Sources;
                }
                (Char('d'), &KeyModifiers::NONE) => {
                    app.mode = Mode::Clients;
                }
                _ => {}
            }
        }
    }

    fn get_help() -> Option<Vec<(&'static str, &'static str)>> {
        Some(vec![
            ("Enter", "Confirm"),
            ("q", "Exit App"),
            ("g", "Top"),
            ("G", "Bottom"),
            ("j, ↓", "Down"),
            ("k, ↑", "Up"),
            ("n, l, →", "Next Page"),
            ("p, h, ←", "Prev Page"),
            ("N, L", "Last Page"),
            ("P, H", "First Page"),
            ("r", "Reload"),
            ("/, i", "Search"),
            ("c", "Categories"),
            ("f", "Filters"),
            ("s", "Sort"),
            ("S", "Sort reversed"),
            ("t", "Themes"),
            ("d", "Select download client"),
            ("Ctrl-p", "Goto page"),
            ("Ctrl-s", "Select source"),
        ])
    }
}
