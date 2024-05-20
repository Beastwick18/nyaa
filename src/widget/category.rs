use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Margin, Rect},
    style::{Color, Style, Stylize as _},
    text::{Line, Text},
    widgets::{Row, ScrollbarOrientation, StatefulWidget, Table},
    Frame,
};

use crate::{
    app::{Context, LoadType, Mode},
    style, title,
};

use super::{border_block, VirtualStatefulTable, Widget};

#[derive(Clone)]
pub struct CatEntry {
    pub name: String,
    pub cfg: String,
    pub id: usize,
    pub icon: CatIcon,
}

#[derive(Clone)]
pub struct CatIcon {
    pub label: &'static str,
    pub color: Color,
}

impl Default for CatIcon {
    fn default() -> Self {
        CatIcon {
            label: "???",
            color: Color::Gray,
        }
    }
}

impl CatEntry {
    pub fn new(name: &str, cfg: &str, id: usize, label: &'static str, color: Color) -> Self {
        CatEntry {
            name: name.to_string(),
            cfg: cfg.to_string(),
            id,
            icon: CatIcon { label, color },
        }
    }
}

#[derive(Clone)]
pub struct CatStruct {
    pub name: String,
    pub entries: Vec<CatEntry>,
}

#[derive(Default)]
pub struct CategoryPopup {
    // pub category: usize,
    pub major: usize,
    pub minor: usize,
    pub max_cat: usize,
    pub table: VirtualStatefulTable,
}

impl CategoryPopup {
    fn next_tab(&mut self) {
        self.major = match self.major + 1 >= self.max_cat {
            true => 0,
            false => self.major + 1,
        };
        self.minor = 0;
        if self.table.state.offset() > self.major {
            *self.table.state.offset_mut() = self.major;
        }
    }

    fn prev_tab(&mut self) {
        self.major = match self.major == 0 {
            true => self.max_cat - 1,
            false => self.major - 1,
        };
        self.minor = 0;
        if self.table.state.offset() > self.major {
            *self.table.state.offset_mut() = self.major;
        }
    }
}

impl Widget for CategoryPopup {
    fn draw(&mut self, f: &mut Frame, ctx: &Context, area: Rect) {
        self.max_cat = ctx.src_info.cats.len(); // TODO: Bad
        if let Some(cat) = ctx.src_info.cats.get(self.major) {
            let mut tbl: Vec<Row> = ctx
                .src_info
                .cats
                .iter()
                .enumerate()
                .map(|(i, e)| match i == self.major {
                    false => Row::new(Text::raw(format!(" ▶ {}", e.name))),
                    true => Row::new(Text::raw(format!(" ▼ {}", e.name)))
                        .style(style!(bg:ctx.theme.solid_bg, fg:ctx.theme.solid_fg)),
                })
                .collect();

            let cat_rows = cat.entries.iter().map(|e| {
                Row::new(vec![Line::from(vec![
                    match e.id == ctx.category {
                        true => "  ",
                        false => "   ",
                    }
                    .into(),
                    e.icon.label.fg(e.icon.color),
                    " ".into(),
                    e.name.to_owned().into(),
                ])])
                // match i == self.minor {
                //     true => row.bg(ctx.theme.hl_bg),
                //     false => row,
                // }
            });
            // self.table.select(self.major + self.minor + 1);
            self.table.scrollbar_state = self
                .table
                .scrollbar_state
                .content_length(cat.entries.len() + ctx.src_info.cats.len());
            // let last_elem = self.major + cat.entries.len() + 1;
            // if last_elem > center.height - 2 {
            //
            // }
            // *self.table.state.offset_mut() = 0;
            // self.table.scrollbar_state = self.table.scrollbar_state.position(0);

            tbl.splice(self.major + 1..self.major + 1, cat_rows);

            let center = super::centered_rect(33, 14, area);
            let clear = super::centered_rect(center.width + 2, center.height, area);
            super::clear(clear, f.buffer_mut(), ctx.theme.bg);
            let table = Table::new(tbl, [Constraint::Percentage(100)])
                .block(border_block(&ctx.theme, true).title(title!("Category")))
                .highlight_style(Style::default().bg(ctx.theme.hl_bg));
            StatefulWidget::render(table, center, f.buffer_mut(), &mut self.table.state);

            let sb = super::scrollbar(ctx, ScrollbarOrientation::VerticalRight);
            let sb_area = center.inner(&Margin {
                vertical: 1,
                horizontal: 0,
            });
            StatefulWidget::render(sb, sb_area, f.buffer_mut(), &mut self.table.scrollbar_state);
            // .render(center, f.buffer_mut());
        }
    }

    fn handle_event(&mut self, ctx: &mut Context, e: &Event) {
        if let Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            ..
        }) = e
        {
            match code {
                KeyCode::Enter => {
                    if let Some(cat) = ctx.src_info.cats.get(self.major) {
                        if let Some(item) = cat.entries.get(self.minor) {
                            ctx.category = item.id;
                            ctx.notify(format!("Category \"{}\"", item.name));
                        }
                    }
                    ctx.mode = Mode::Loading(LoadType::Categorizing);
                }
                KeyCode::Esc | KeyCode::Char('c') | KeyCode::Char('q') => {
                    ctx.mode = Mode::Normal;
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    if let Some(cat) = ctx.src_info.cats.get(self.major) {
                        self.minor = match self.minor + 1 >= cat.entries.len() {
                            true => {
                                self.next_tab();
                                0
                            }
                            false => self.minor + 1,
                        };
                        self.table.select(self.major + self.minor + 1);
                    }
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    if ctx.src_info.cats.get(self.major).is_some() {
                        self.minor = match self.minor < 1 {
                            true => {
                                self.prev_tab();
                                match ctx.src_info.cats.get(self.major) {
                                    Some(cat) => cat.entries.len() - 1,
                                    None => 0,
                                }
                            }
                            false => self.minor - 1,
                        };
                        self.table.select(self.major + self.minor + 1);
                    }
                }
                KeyCode::Char('G') => {
                    if let Some(cat) = ctx.src_info.cats.get(self.major) {
                        self.minor = cat.entries.len() - 1;
                        self.table.select(self.major + self.minor + 1);
                    }
                }
                KeyCode::Char('g') => {
                    self.minor = 0;
                    self.table.select(self.major + self.minor + 1);
                }
                KeyCode::Tab | KeyCode::Char('J') => {
                    self.next_tab();
                    self.table.select(self.major + self.minor + 1);
                }
                KeyCode::BackTab | KeyCode::Char('K') => {
                    self.prev_tab();
                    self.table.select(self.major + self.minor + 1);
                }
                _ => {}
            }
        }
    }

    fn get_help() -> Option<Vec<(&'static str, &'static str)>> {
        Some(vec![
            ("Enter", "Confirm"),
            ("Esc, c, q", "Close"),
            ("j, ↓", "Down"),
            ("k, ↑", "Up"),
            ("g", "Top"),
            ("G", "Bottom"),
            ("Tab, J", "Next Tab"),
            ("S-Tab, K", "Prev Tab"),
        ])
    }
}
