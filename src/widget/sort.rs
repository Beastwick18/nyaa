use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Rect},
    widgets::{Row, StatefulWidget as _, Table},
    Frame,
};

use crate::{
    app::{Context, LoadType, Mode},
    style, title,
};

use super::{border_block, VirtualStatefulTable, Widget};

#[derive(Clone, Copy)]
pub struct SelectedSort {
    pub sort: usize,
    pub dir: SortDir,
}

impl Default for SelectedSort {
    fn default() -> Self {
        Self {
            sort: 0,
            dir: SortDir::Desc,
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum SortDir {
    Desc,
    Asc,
}

impl SortDir {
    pub fn to_url(self) -> String {
        match self {
            SortDir::Desc => "desc",
            SortDir::Asc => "asc",
        }
        .to_owned()
    }
}

pub struct SortPopup {
    pub table: VirtualStatefulTable,
    pub selected: SelectedSort,
}

impl Default for SortPopup {
    fn default() -> Self {
        SortPopup {
            table: VirtualStatefulTable::new(),
            selected: SelectedSort::default(),
        }
    }
}

impl Widget for SortPopup {
    fn draw(&mut self, f: &mut Frame, ctx: &Context, area: Rect) {
        let buf = f.buffer_mut();
        let center = super::centered_rect(30, ctx.src_info.sorts.len() as u16 + 2, area);
        // let clear = super::centered_rect(center.width + 2, center.height, area);
        let items = ctx.src_info.sorts.iter().enumerate().map(|(i, item)| {
            Row::new([match i == self.selected.sort {
                true => format!("  {}", item),
                false => format!("   {}", item),
            }])
        });
        let table = Table::new(items, [Constraint::Percentage(100)])
            .block(border_block(&ctx.theme, true).title(title!(match ctx.mode
                == Mode::Sort(SortDir::Asc)
            {
                true => "Sort Ascending",
                false => "Sort Descending",
            })))
            .highlight_style(style!(bg:ctx.theme.hl_bg));
        super::clear(center, buf, ctx.theme.bg);
        table.render(center, buf, &mut self.table.state);
    }

    fn handle_event(&mut self, ctx: &mut Context, e: &Event) {
        if let Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            ..
        }) = e
        {
            match code {
                KeyCode::Esc | KeyCode::Char('s') | KeyCode::Char('q') => {
                    ctx.mode = Mode::Normal;
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    self.table.next_wrap(ctx.src_info.sorts.len(), 1);
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.table.next_wrap(ctx.src_info.sorts.len(), -1);
                }
                KeyCode::Char('G') => {
                    self.table.select(ctx.src_info.sorts.len() - 1);
                }
                KeyCode::Char('g') => {
                    self.table.select(0);
                }
                KeyCode::Enter => {
                    if let Some(i) = self.table.state.selected() {
                        self.selected.sort = i;
                        self.selected.dir = match ctx.mode == Mode::Sort(SortDir::Asc) {
                            true => SortDir::Asc,
                            false => SortDir::Desc,
                        };
                        ctx.mode = Mode::Loading(LoadType::Sorting);
                        if let Some(s) = ctx.src_info.sorts.get(i) {
                            ctx.notify(format!("Sort by \"{}\"", s));
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn get_help() -> Option<Vec<(&'static str, &'static str)>> {
        Some(vec![
            ("Enter", "Confirm"),
            ("Esc, s, q", "Close"),
            ("j, ↓", "Down"),
            ("k, ↑", "Up"),
            ("g", "Top"),
            ("G", "Bottom"),
        ])
    }
}
