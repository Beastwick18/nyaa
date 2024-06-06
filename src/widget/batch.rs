use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use human_bytes::human_bytes;
use ratatui::{
    layout::{Constraint, Margin, Rect},
    style::{Style, Stylize},
    widgets::{Clear, Row, ScrollbarOrientation, StatefulWidget, Table, Widget},
    Frame,
};

use crate::{
    app::{Context, LoadType, Mode},
    source::ItemType,
    title,
};

use super::{border_block, Corner, VirtualStatefulTable};

pub struct BatchWidget {
    table: VirtualStatefulTable,
}

impl Default for BatchWidget {
    fn default() -> Self {
        BatchWidget {
            table: VirtualStatefulTable::new(),
        }
    }
}

impl super::Widget for BatchWidget {
    fn draw(&mut self, f: &mut Frame, ctx: &Context, area: Rect) {
        let buf = f.buffer_mut();
        let block = border_block(&ctx.theme, ctx.mode == Mode::Batch).title(title!("Batch"));
        let focus_color = match ctx.mode {
            Mode::Batch => ctx.theme.border_focused_color,
            _ => ctx.theme.border_color,
        };
        let rows = ctx
            .batch
            .iter()
            .map(|i| {
                Row::new([
                    i.icon.label.fg((i.icon.color)(&ctx.theme)),
                    i.title.to_owned().fg(match i.item_type {
                        ItemType::Trusted => ctx.theme.success,
                        ItemType::Remake => ctx.theme.error,
                        ItemType::None => ctx.theme.fg,
                    }),
                    format!("{:>9}", i.size).fg(ctx.theme.fg),
                ])
            })
            .collect::<Vec<Row>>();

        let header = ["Cat", "Name", "  Size"];
        let header = Row::new(header)
            .fg(focus_color)
            .underlined()
            .height(1)
            .bottom_margin(0);
        let table = Table::new(
            rows.to_owned(),
            [
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Length(9),
            ],
        )
        .block(block)
        .header(header)
        .highlight_style(Style::default().bg(ctx.theme.hl_bg));
        Clear.render(area, buf);

        let num_items = rows.len();
        super::scroll_padding(
            self.table.selected().unwrap_or(0),
            area.height as usize,
            3,
            num_items,
            ctx.config.scroll_padding,
            self.table.state.offset_mut(),
        );

        StatefulWidget::render(table, area, buf, &mut self.table.state);
        if ctx.batch.len() + 2 > area.height as usize {
            let sb = super::scrollbar(ctx, ScrollbarOrientation::VerticalRight);
            let sb_area = area.inner(&Margin {
                vertical: 1,
                horizontal: 0,
            });
            StatefulWidget::render(
                sb,
                sb_area,
                buf,
                &mut self.table.scrollbar_state.content_length(rows.len()),
            );
        }

        let size = human_bytes(ctx.batch.iter().fold(0, |acc, i| acc + i.bytes) as f64);
        let right_str = title!("Size({}): {}", ctx.batch.len(), size);
        if let Some((tr, area)) = Corner::TopRight.try_title(right_str, area, true) {
            tr.render(area, buf);
        }
    }

    fn handle_event(&mut self, ctx: &mut Context, evt: &Event) {
        if let Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            modifiers,
            ..
        }) = evt
        {
            use KeyCode::*;
            match (code, modifiers) {
                (Esc | Tab | BackTab, _) => {
                    ctx.mode = Mode::Normal;
                }
                (Char('q'), &KeyModifiers::NONE) => {
                    ctx.quit();
                }
                (Char('j') | Down, &KeyModifiers::NONE) => {
                    self.table.next(ctx.batch.len(), 1);
                }
                (Char('k') | Up, &KeyModifiers::NONE) => {
                    self.table.next(ctx.batch.len(), -1);
                }
                (Char('J'), &KeyModifiers::SHIFT) => {
                    self.table.next(ctx.batch.len(), 4);
                }
                (Char('K'), &KeyModifiers::SHIFT) => {
                    self.table.next(ctx.batch.len(), -4);
                }
                (Char('g'), &KeyModifiers::NONE) => {
                    self.table.select(0);
                }
                (Char('G'), &KeyModifiers::SHIFT) => {
                    self.table.select(ctx.batch.len() - 1);
                }
                (Char(' '), &KeyModifiers::NONE) => {
                    if let Some(i) = self.table.selected() {
                        self.table.next(ctx.batch.len(), 0);
                        ctx.batch.remove(i);
                        self.table.next(ctx.batch.len(), 0);
                    }
                }
                (Char('a'), &KeyModifiers::CONTROL) => {
                    ctx.mode = Mode::Loading(LoadType::Batching);
                }
                (Char('x'), &KeyModifiers::CONTROL) => {
                    ctx.batch.clear();
                }
                _ => {}
            };
        }
    }

    fn get_help() -> Option<Vec<(&'static str, &'static str)>> {
        Some(vec![
            ("Enter", "Download single torrent"),
            ("Ctrl-A", "Download all torrents"),
            ("Ctrl-X", "Clear batch"),
            ("Esc/Tab/Shift-Tab", "Back to results"),
            ("q", "Exit app"),
            ("g/G", "Goto Top/Bottom"),
            ("k, ↑", "Up"),
            ("j, ↓", "Down"),
            ("K, J", "Up/Down 4 items"),
            ("Space", "Toggle item for batch download"),
        ])
    }
}
