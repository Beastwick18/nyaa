use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Constraint, Margin, Rect},
    style::{Style, Stylize},
    widgets::{Clear, Paragraph, Row, ScrollbarOrientation, StatefulWidget, Table, Widget},
    Frame,
};
use unicode_width::UnicodeWidthStr as _;

use crate::app::{Context, LoadType, Mode};

use super::{border_block, VirtualStatefulTable};

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
        let block = border_block(&ctx.theme, ctx.mode == Mode::Batch).title("Batch");
        let rows = ctx
            .batch
            .iter()
            .map(|i| {
                Row::new([i.title.to_owned()]).fg(if i.trusted {
                    ctx.theme.trusted
                } else if i.remake {
                    ctx.theme.remake
                } else {
                    ctx.theme.fg
                })
            })
            .collect::<Vec<Row>>();
        let table = Table::new(rows.to_owned(), [Constraint::Percentage(100)])
            .block(block)
            .highlight_style(Style::default().bg(ctx.theme.hl_bg));
        Clear.render(area, buf);
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

        let right_str = format!("Total: {}", ctx.batch.len());
        let text = Paragraph::new(right_str.clone());
        let right = Rect::new(
            area.right() - 1 - right_str.width() as u16,
            area.top(),
            right_str.width() as u16,
            1,
        );
        f.render_widget(text, right);
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
                _ => {}
            };
        }
    }

    fn get_help() -> Option<Vec<(&'static str, &'static str)>> {
        Some(vec![
            ("Enter", "Download single torrent"),
            ("Ctrl-A", "Download all torrents"),
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
