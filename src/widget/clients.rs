use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Rect},
    widgets::{Row, StatefulWidget as _, Table},
    Frame,
};

use crate::{
    app::{Context, Mode},
    client::Client,
    style,
};

use super::{border_block, EnumIter, StatefulTable, Widget};

pub struct ClientsPopup {
    pub table: StatefulTable<String>,
}

impl Default for ClientsPopup {
    fn default() -> Self {
        ClientsPopup {
            table: StatefulTable::new(Client::iter().map(|item| item.to_string()).collect()),
        }
    }
}

impl Widget for ClientsPopup {
    fn draw(&mut self, f: &mut Frame, ctx: &Context, area: Rect) {
        let buf = f.buffer_mut();
        let center = super::centered_rect(30, self.table.items.len() as u16 + 2, area);
        let clear = super::centered_rect(center.width + 2, center.height, area);
        let items = self.table.items.iter().enumerate().map(|(i, item)| {
            Row::new(vec![match i == ctx.client.to_owned() as usize {
                true => format!("  {}", item.to_owned()),
                false => format!("   {}", item.to_owned()),
            }])
        });
        super::clear(clear, buf, ctx.theme.bg);
        let table = Table::new(items, [Constraint::Percentage(100)])
            .block(border_block(ctx.theme, true).title("Download Client"))
            .highlight_style(style!(bg:ctx.theme.hl_bg));
        table.render(center, buf, &mut self.table.state);
    }

    fn handle_event(&mut self, app: &mut Context, e: &Event) {
        if let Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            ..
        }) = e
        {
            match code {
                KeyCode::Esc | KeyCode::Char('d') | KeyCode::Char('q') => {
                    app.mode = Mode::Normal;
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
                    if let Some(c) = Client::iter().nth(self.table.state.selected().unwrap_or(0)) {
                        app.client = *c;
                        match c.load_config(app) {
                            Ok(_) => app.notify(format!(
                                "Updated download client to \"{}\"",
                                c.to_string()
                            )),
                            Err(e) => app.show_error(format!(
                                "Failed to update download client in config file:\n{}",
                                e
                            )),
                        };
                        app.mode = Mode::Normal;
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
