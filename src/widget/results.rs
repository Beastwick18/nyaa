use std::cmp::max;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Margin, Rect},
    style::{Style, Stylize},
    symbols::{self},
    text::Line,
    widgets::{Clear, Paragraph, Row, ScrollbarOrientation, StatefulWidget, Table, Widget},
    Frame,
};
use serde::{Deserialize, Serialize};

use crate::{
    app::{Context, LoadType, Mode},
    title,
    widget::sort::SortDir,
};

use super::{border_block, centered_rect, TitlePosition, VirtualStatefulTable};

#[derive(Clone, Copy, Serialize, Deserialize, Default)]
pub struct ColumnsConfig {
    category: Option<bool>,
    title: Option<bool>,
    size: Option<bool>,
    date: Option<bool>,
    seeders: Option<bool>,
    leechers: Option<bool>,
    downloads: Option<bool>,
}

// impl ColumnsConfig {
//     fn array(self) -> [bool; 7] {
//         [
//             self.category.unwrap_or(true),
//             self.title.unwrap_or(true),
//             self.size.unwrap_or(true),
//             self.date.unwrap_or(true),
//             self.seeders.unwrap_or(true),
//             self.leechers.unwrap_or(true),
//             self.downloads.unwrap_or(true),
//         ]
//     }
// }

pub struct ResultsWidget {
    pub table: VirtualStatefulTable,
    control_space: bool,
}

impl ResultsWidget {
    fn try_select_toggle(&self, ctx: &mut Context) {
        if let Some(sel) = self.table.state.selected() {
            if let Some(item) = ctx.results.items.get(sel) {
                if let Some(p) = ctx.batch.iter().position(|s| s.id == item.id) {
                    ctx.batch.remove(p);
                } else {
                    ctx.batch.push(item.to_owned());
                }
            }
        }
    }

    fn try_select(&self, ctx: &mut Context) {
        if let Some(sel) = self.table.state.selected() {
            if let Some(item) = ctx.results.items.get(sel) {
                if !ctx.batch.iter().any(|s| s.id == item.id) {
                    ctx.batch.push(item.to_owned());
                }
            }
        }
    }
}

impl Default for ResultsWidget {
    fn default() -> Self {
        ResultsWidget {
            table: VirtualStatefulTable::new(),
            // date_width: 6,
            // raw_date_width: 4,
            control_space: false,
        }
    }
}

impl super::Widget for ResultsWidget {
    fn draw(&mut self, f: &mut Frame, ctx: &Context, area: Rect) {
        let buf = f.buffer_mut();
        let focus_color = match ctx.mode {
            Mode::Normal | Mode::KeyCombo(_) => ctx.theme.border_focused_color,
            _ => ctx.theme.border_color,
        };
        let header: Row = ctx.results.headers.clone().into();
        let header = header.fg(focus_color).underlined();

        Clear.render(area, buf);
        let items: Vec<Row> = match ctx.mode {
            Mode::Loading(loadtype) => {
                let message = match loadtype {
                    LoadType::Sourcing => "Sourcing…",
                    LoadType::Searching => "Searching…",
                    LoadType::Sorting => "Sorting…",
                    LoadType::Filtering => "Filtering…",
                    LoadType::Categorizing => "Categorizing…",
                    LoadType::Batching => "Downloading batch…",
                    LoadType::Downloading => "Downloading…",
                };
                let load_area = centered_rect(message.len() as u16, 1, area);
                Paragraph::new(message).render(load_area, buf);
                vec![]
            }
            _ => ctx
                .results
                .rows
                .clone()
                .into_iter()
                .map(Into::into)
                .collect(),
        };

        let sb = super::scrollbar(ctx, ScrollbarOrientation::VerticalRight).begin_symbol(Some(""));
        let sb_area = area.inner(&Margin {
            vertical: 1,
            horizontal: 0,
        });

        let num_items = items.len();
        let first_item = (ctx.page - 1) * 75;
        let focused = matches!(ctx.mode, Mode::Normal | Mode::KeyCombo(_));
        let table = Table::new(items, ctx.results.binding.to_owned())
            .header(header)
            .block(border_block(&ctx.theme, focused).title(title!(
                "Results {}-{} ({} total): Page {}/{}",
                first_item + 1,
                num_items + first_item,
                ctx.total_results,
                ctx.page,
                ctx.last_page,
            )))
            .highlight_style(Style::default().bg(ctx.theme.hl_bg));
        StatefulWidget::render(table, area, buf, &mut self.table.state);
        StatefulWidget::render(sb, sb_area, buf, &mut self.table.scrollbar_state);

        if !matches!(ctx.mode, Mode::Loading(_)) && num_items == 0 {
            let center = centered_rect(10, 1, area);
            Paragraph::new("No results").render(center, buf);
        }

        if let Some(visible_items) = ctx.results.items.get(self.table.state.offset()..) {
            let selected_ids: Vec<usize> = ctx.batch.iter().map(|i| i.id).collect();
            let vert_left = ctx.theme.border.to_border_set().vertical_left;
            let lines = visible_items
                .iter()
                .map(|i| {
                    Line::from(
                        match selected_ids.contains(&i.id) {
                            true => symbols::border::QUADRANT_BLOCK,
                            false => vert_left,
                        }
                        .to_owned(),
                    )
                })
                .collect::<Vec<Line>>();
            let para = Paragraph::new(lines);
            let pararea = Rect::new(area.x, area.y + 2, 1, area.height - 3);
            para.render(pararea, buf);
        }

        let dl_src = title!(
            "dl: {}, src: {}",
            ctx.client.to_string(),
            ctx.src.to_string()
        );
        if let Some((tr, area)) = TitlePosition::TopRight.try_widget(dl_src, area) {
            f.render_widget(tr, area);
        }

        if !ctx.last_key.is_empty() {
            let key_str = title!(ctx.last_key);
            if let Some((br, area)) = TitlePosition::BottomRight.try_widget(key_str, area) {
                f.render_widget(br, area);
            }
        }

        if let Some(notif) = ctx.notification.clone() {
            if let Some((bl, area)) = TitlePosition::BottomLeft.try_widget(notif, area) {
                f.render_widget(bl, area);
            }
        }
    }

    fn handle_event(&mut self, ctx: &mut Context, e: &Event) {
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
                    ctx.mode = Mode::Category;
                }
                (Char('s'), &KeyModifiers::NONE) => {
                    ctx.mode = Mode::Sort(SortDir::Desc);
                }
                (Char('S'), &KeyModifiers::SHIFT) => {
                    ctx.mode = Mode::Sort(SortDir::Asc);
                }
                (Char('f'), &KeyModifiers::NONE) => {
                    ctx.mode = Mode::Filter;
                }
                (Char('t'), &KeyModifiers::NONE) => {
                    ctx.mode = Mode::Theme;
                }
                (Char('/') | Char('i'), &KeyModifiers::NONE) => {
                    ctx.mode = Mode::Search;
                }
                (Char('p'), &KeyModifiers::CONTROL) => {
                    ctx.mode = Mode::Page;
                }
                (Char('p') | Char('h') | Left, &KeyModifiers::NONE) => {
                    if ctx.page > 1 {
                        ctx.page -= 1;
                        ctx.mode = Mode::Loading(LoadType::Searching);
                    }
                }
                (Char('n') | Char('l') | Right, &KeyModifiers::NONE) => {
                    if ctx.page < ctx.last_page {
                        ctx.page += 1;
                        ctx.mode = Mode::Loading(LoadType::Searching);
                    }
                }
                (Char('r'), &KeyModifiers::NONE) => {
                    ctx.mode = Mode::Loading(LoadType::Searching);
                }
                (Char('q'), &KeyModifiers::NONE) => {
                    ctx.quit();
                }
                (Char('j') | KeyCode::Down, &KeyModifiers::NONE) => {
                    if self
                        .table
                        .state
                        .selected()
                        .is_some_and(|s| s + 1 != ctx.results.items.len())
                    {
                        self.table.next(ctx.results.items.len(), 1);
                        if self.control_space {
                            self.try_select_toggle(ctx);
                        }
                    }
                }
                (Char('k') | KeyCode::Up, &KeyModifiers::NONE) => {
                    if self.table.state.selected().is_some_and(|s| s != 0) {
                        if self.control_space {
                            self.try_select_toggle(ctx);
                        }
                        self.table.next(ctx.results.items.len(), -1);
                    }
                }
                (Char('J'), &KeyModifiers::SHIFT) => {
                    self.table.next(ctx.results.items.len(), 4);
                }
                (Char('K'), &KeyModifiers::SHIFT) => {
                    self.table.next(ctx.results.items.len(), -4);
                }
                (Char('G'), &KeyModifiers::SHIFT) => {
                    self.table.select(max(ctx.results.items.len(), 1) - 1);
                }
                (Char('g'), &KeyModifiers::NONE) => {
                    self.table.select(0);
                }
                (Char('H') | Char('P'), &KeyModifiers::SHIFT) => {
                    if ctx.page != 1 {
                        ctx.page = 1;
                        ctx.mode = Mode::Loading(LoadType::Searching);
                    }
                }
                (Char('L') | Char('N'), &KeyModifiers::SHIFT) => {
                    if ctx.page != ctx.last_page && ctx.last_page > 0 {
                        ctx.page = ctx.last_page;
                        ctx.mode = Mode::Loading(LoadType::Searching);
                    }
                }
                (Enter, &KeyModifiers::NONE) => {
                    ctx.mode = Mode::Loading(LoadType::Downloading);
                }
                (Char('s'), &KeyModifiers::CONTROL) => {
                    ctx.mode = Mode::Sources;
                }
                (Char('d'), &KeyModifiers::NONE) => {
                    ctx.mode = Mode::Clients;
                }
                (Char('u'), &KeyModifiers::NONE) => {
                    ctx.mode = Mode::User;
                }
                (Char('o'), &KeyModifiers::NONE) => {
                    let link = ctx
                        .results
                        .items
                        .get(self.table.state.selected().unwrap_or(0))
                        .map(|item| item.post_link.clone())
                        .unwrap_or("https://nyaa.si".to_owned());
                    let res = open::that_detached(link.clone());
                    if let Err(e) = res {
                        ctx.show_error(format!("Failed to open {}:\n{}", link, e));
                    } else {
                        ctx.notify(format!("Opened {}", link));
                    }
                }
                (Char('y'), &KeyModifiers::NONE) => ctx.mode = Mode::KeyCombo(vec!['y']),
                (Char(' '), &KeyModifiers::CONTROL) => {
                    self.control_space = !self.control_space;
                    if self.control_space {
                        ctx.notify("Entered VISUAL mode");
                        self.try_select(ctx);
                    } else {
                        ctx.notify("Exited VISUAL mode");
                    }
                }
                (Char(' '), &KeyModifiers::NONE) => {
                    if let Some(sel) = self.table.state.selected() {
                        if let Some(item) = &mut ctx.results.items.get_mut(sel) {
                            if let Some(p) = ctx.batch.iter().position(|s| s.id == item.id) {
                                ctx.batch.remove(p);
                            } else {
                                ctx.batch.push(item.to_owned());
                            }
                        }
                    }
                }
                (Tab | BackTab, _) => {
                    ctx.mode = Mode::Batch;
                }
                (Esc, &KeyModifiers::NONE) => {
                    ctx.notification = None;
                    if self.control_space {
                        ctx.notify("Exited VISUAL mode");
                    }
                    self.control_space = false;
                }
                _ => {}
            }
        }
    }

    fn get_help() -> Option<Vec<(&'static str, &'static str)>> {
        Some(vec![
            ("Enter", "Confirm"),
            ("Esc", "Dismiss notification"),
            ("q", "Exit App"),
            ("g/G", "Goto Top/Bottom"),
            ("k, ↑", "Up"),
            ("j, ↓", "Down"),
            ("K, J", "Up/Down 4 items"),
            ("n, l, →", "Next Page"),
            ("p, h, ←", "Prev Page"),
            ("N, L", "Last Page"),
            ("P, H", "First Page"),
            ("r", "Reload"),
            ("o", "Open in browser"),
            ("yt, ym, yp", "Copy torrent/magnet/post link"),
            ("Space", "Toggle item for batch download"),
            ("Ctrl-Space", "Multi-line select torrents"),
            ("Tab/Shift-Tab", "Switch to Batches"),
            ("/, i", "Search"),
            ("c", "Categories"),
            ("f", "Filters"),
            ("s", "Sort"),
            ("S", "Sort reversed"),
            ("t", "Themes"),
            ("u", "Filter by User"),
            ("d", "Select download client"),
            ("Ctrl-p", "Goto page"),
            ("Ctrl-s", "Select source"),
        ])
    }
}
