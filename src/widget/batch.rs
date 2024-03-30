use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Constraint, Margin, Rect},
    style::Stylize,
    widgets::{Clear, Row, Scrollbar, ScrollbarOrientation, StatefulWidget, Table, Widget},
    Frame,
};

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
        let block = border_block(ctx.theme, ctx.mode == Mode::Search).title("Batch");
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
        let table = Table::new(rows.to_owned(), [Constraint::Percentage(100)]).block(block);
        Clear.render(area, buf);
        StatefulWidget::render(table, area, buf, &mut self.table.state);
        if ctx.batch.len() + 2 > area.height as usize {
            let sb = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .track_symbol(Some("│"))
                .begin_symbol(None)
                .end_symbol(None);
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
                (Esc, &KeyModifiers::NONE) => {
                    ctx.mode = Mode::Normal;
                }
                (Enter, &KeyModifiers::NONE) => {
                    ctx.mode = Mode::Loading(LoadType::Searching);
                    ctx.page = 1; // Go back to first page
                }
                _ => {}
            };
        }
    }

    fn get_help() -> Option<Vec<(&'static str, &'static str)>> {
        Some(vec![("Enter", "Confirm"), ("Esc", "Stop")])
    }
}
