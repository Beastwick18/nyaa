use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Rect},
    widgets::{Row, StatefulWidget as _, Table},
    Frame,
};
use strum::VariantArray;

use crate::{
    app::{Context, Mode},
    client::Client,
    style, title,
};

use super::{border_block, StatefulTable, Widget};

pub struct ClientsPopup {
    pub table: StatefulTable<Client>,
}

impl Default for ClientsPopup {
    fn default() -> Self {
        ClientsPopup {
            table: StatefulTable::new(Client::VARIANTS),
        }
    }
}

impl Widget for ClientsPopup {
    fn draw(&mut self, f: &mut Frame, ctx: &Context, area: Rect) {
        let buf = f.buffer_mut();
        let center = super::centered_rect(30, self.table.items.len() as u16 + 2, area);
        let items = self.table.items.iter().map(|item| {
            Row::new(vec![match item == &ctx.client {
                true => format!("  {}", item),
                false => format!("   {}", item),
            }])
        });
        super::clear(center, buf, ctx.theme.bg);
        let table = Table::new(items, [Constraint::Percentage(100)])
            .block(border_block(&ctx.theme, true).title(title!("Download Client")))
            .highlight_style(style!(bg:ctx.theme.hl_bg));
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
                KeyCode::Esc | KeyCode::Char('d') | KeyCode::Char('q') => {
                    ctx.mode = Mode::Normal;
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    self.table.next_wrap(1);
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.table.next_wrap(-1);
                }
                KeyCode::Char('G') => {
                    self.table.select(self.table.items.len() - 1);
                }
                KeyCode::Char('g') => {
                    self.table.select(0);
                }
                KeyCode::Enter => {
                    if let Some(c) = self.table.selected() {
                        ctx.client = *c;

                        c.load_config(ctx);
                        match ctx.save_config() {
                            Ok(_) => ctx.notify(format!("Updated download client to \"{}\"", c)),
                            Err(e) => ctx.show_error(format!("Failed to update config:\n{}", e)),
                        }
                        ctx.mode = Mode::Normal;
                    }
                }
                _ => {}
            }
        }
    }

    fn get_help() -> Option<Vec<(&'static str, &'static str)>> {
        Some(vec![
            ("Enter", "Confirm"),
            ("Esc, d, q", "Close"),
            ("j, ↓", "Down"),
            ("k, ↑", "Up"),
            ("g", "Top"),
            ("G", "Bottom"),
        ])
    }
}
