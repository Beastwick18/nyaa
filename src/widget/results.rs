use std::{
    cmp::max,
    io::{BufReader, Read},
    process::{Command, Stdio},
};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Constraint, Margin, Rect},
    style::{Modifier, Style, Stylize},
    text::Text,
    widgets::{Clear, Paragraph, Row, Scrollbar, ScrollbarOrientation, Table},
    Frame,
};

use crate::{
    app::{App, LoadType, Mode},
    source::nyaa_html::Item,
};

use super::{centered_rect, create_block, StatefulTable};

pub struct ResultsWidget {
    pub table: StatefulTable<Item>,
}

impl ResultsWidget {
    pub fn with_items(&mut self, items: Vec<Item>) {
        let len = items.len();
        self.table.items = items;
        self.table.select(0);
        self.table.scrollbar_state = self.table.scrollbar_state.content_length(len);
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
        let binding = Constraint::from_lengths([3, area.width - 32, 9, 4, 4, 5]);
        let header = Row::new([
            "Cat".to_owned(),
            "Name".to_owned(),
            format!("{:^9}", " Size"),
            format!("{:^4}", ""),
            format!("{:^4}", ""),
            format!("{:^4}", ""),
        ])
        .add_modifier(Modifier::BOLD)
        .add_modifier(Modifier::UNDERLINED)
        .fg(focus_color)
        .height(1)
        .bottom_margin(0);

        f.render_widget(Clear, area);
        let items: Vec<Row> = match app.mode {
            Mode::Loading(_) => {
                let area = centered_rect(8, 1, f.size());
                f.render_widget(Paragraph::new("Loading…"), area);
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
                create_block(app.theme, app.mode == Mode::Normal).title(format!(
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
        f.render_stateful_widget(table, area, &mut self.table.state.to_owned());
        f.render_stateful_widget(sb, sb_area, &mut self.table.scrollbar_state.to_owned());

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
                    app.mode = Mode::Sort;
                    app.reverse = false;
                }
                (Char('S'), &KeyModifiers::SHIFT) => {
                    app.mode = Mode::Sort;
                    app.reverse = true;
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
                    let item = match self
                        .table
                        .state
                        .selected()
                        .and_then(|i| self.table.items.get(i))
                    {
                        Some(i) => i,
                        None => return,
                    };
                    let cmd_str = app
                        .config
                        .torrent_client_cmd
                        .replace("{magnet}", &shellwords::escape(item.magnet_link.as_str()))
                        .replace("{torrent}", &shellwords::escape(item.torrent_link.as_str()))
                        .replace("{title}", &shellwords::escape(item.title.as_str()))
                        .replace("{file}", &shellwords::escape(item.file_name.as_str()));
                    let cmd = match shellwords::split(&cmd_str) {
                        Ok(cmd) => cmd,
                        Err(e) => {
                            app.errors.push(format!(
                                "{}\n{}:\nfailed to split command:\n{}",
                                cmd_str, app.config.torrent_client_cmd, e
                            ));
                            return;
                        }
                    };
                    if let [exec, args @ ..] = cmd.as_slice() {
                        let cmd = Command::new(exec)
                            .args(args)
                            .stdin(Stdio::null())
                            .stdout(Stdio::null())
                            .stderr(Stdio::piped())
                            .spawn();
                        let child = match cmd {
                            Ok(child) => child,
                            Err(e) => {
                                app.errors
                                    .push(format!("{}:\nFailed to run:\n{}", cmd_str, e));
                                return;
                            }
                        };
                        let output = match child.wait_with_output() {
                            Ok(output) => output,
                            Err(e) => {
                                app.errors
                                    .push(format!("{}:\nFailed to get output:\n{}", cmd_str, e));
                                return;
                            }
                        };

                        if output.status.code() != Some(0) {
                            let mut err = BufReader::new(&*output.stderr);
                            let mut err_str = String::new();
                            err.read_to_string(&mut err_str).unwrap_or(0);
                            app.errors.push(format!(
                                "{}:\nExited with status code {}:\n{}",
                                cmd_str, output.status, err_str
                            ));
                        }
                    } else {
                        app.errors
                            .push(format!("{}:\nThe command is not valid.", cmd_str));
                    }
                }
                (Char('s'), &KeyModifiers::CONTROL) => {
                    app.mode = Mode::Sources;
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
            ("N, L, →", "Last Page"),
            ("P, H, ←", "First Page"),
            ("r", "Reload"),
            ("/, i", "Search"),
            ("c", "Categories"),
            ("f", "Filters"),
            ("s", "Sort"),
            ("S", "Sort reversed"),
            ("t", "Themes"),
            ("Ctrl-p", "Goto page"),
            ("Ctrl-s", "Select source"),
        ])
    }
}
