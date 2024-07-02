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

pub struct FilterPopup {
    pub table: VirtualStatefulTable,
    pub selected: usize,
}

impl Default for FilterPopup {
    fn default() -> Self {
        FilterPopup {
            table: VirtualStatefulTable::new(),
            selected: 0,
        }
    }
}

impl Widget for FilterPopup {
    fn draw(&mut self, f: &mut Frame, ctx: &Context, area: Rect) {
        let center = super::centered_rect(30, ctx.src_info.filters.len() as u16 + 2, area);
        let items =
            ctx.src_info
                .filters
                .iter()
                .enumerate()
                .map(|(i, item)| match i == self.selected {
                    true => Row::new(vec![format!("  {}", item.to_owned())]),
                    false => Row::new(vec![format!("   {}", item.to_owned())]),
                });
        // super::dim_buffer(area, f.buffer_mut(), 0.5);
        super::clear(center, f.buffer_mut(), ctx.theme.bg);
        Table::new(items, [Constraint::Percentage(100)])
            .block(border_block(&ctx.theme, true).title(title!("Filter")))
            .highlight_style(style!(bg:ctx.theme.hl_bg))
            .render(center, f.buffer_mut(), &mut self.table.state);
    }

    fn handle_event(&mut self, ctx: &mut Context, e: &Event) {
        if let Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            ..
        }) = e
        {
            match code {
                KeyCode::Esc | KeyCode::Char('f') | KeyCode::Char('q') => {
                    ctx.mode = Mode::Normal;
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    self.table.next_wrap(ctx.src_info.filters.len(), 1);
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.table.next_wrap(ctx.src_info.filters.len(), -1);
                }
                KeyCode::Char('G') => {
                    self.table.select(ctx.src_info.filters.len() - 1);
                }
                KeyCode::Char('g') => {
                    self.table.select(0);
                }
                KeyCode::Enter => {
                    if let Some(i) = self.table.state.selected() {
                        self.selected = i;
                        ctx.mode = Mode::Loading(LoadType::Filtering);
                        if let Some(f) = ctx.src_info.filters.get(i) {
                            ctx.notify_info(format!("Filter by \"{}\"", f));
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
            ("Esc, f, q", "Close"),
            ("g", "Top"),
            ("G", "Bottom"),
            ("j, ↓", "Down"),
            ("k, ↑", "Up"),
        ])
    }
}
