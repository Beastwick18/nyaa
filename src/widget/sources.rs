use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Rect},
    widgets::{Row, StatefulWidget as _, Table},
    Frame,
};

use crate::{
    app::{Context, LoadType, Mode},
    source::Sources,
    style,
};

use super::{border_block, EnumIter, StatefulTable, Widget};

pub struct SourcesPopup {
    pub table: StatefulTable<String>,
}

impl Default for SourcesPopup {
    fn default() -> Self {
        SourcesPopup {
            table: StatefulTable::new(Sources::iter().map(|item| item.to_string()).collect()),
        }
    }
}

impl Widget for SourcesPopup {
    fn draw(&mut self, f: &mut Frame, app: &Context, area: Rect) {
        let buf = f.buffer_mut();
        let center = super::centered_rect(30, self.table.items.len() as u16 + 2, area);
        let clear = super::centered_rect(center.width + 2, center.height, area);
        let items = self.table.items.iter().enumerate().map(|(i, item)| {
            Row::new(vec![match i == app.src.to_owned() as usize {
                true => format!("  {}", item.to_owned()),
                false => format!("   {}", item.to_owned()),
            }])
        });
        super::clear(clear, buf, app.theme.bg);
        let table = Table::new(items, [Constraint::Percentage(100)])
            .block(border_block(app.theme, true).title("Source"))
            .highlight_style(style!(bg:app.theme.hl_bg));
        table.render(center, buf, &mut self.table.state);
    }

    fn handle_event(&mut self, app: &mut crate::app::Context, e: &crossterm::event::Event) {
        if let Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            ..
        }) = e
        {
            match code {
                KeyCode::Esc | KeyCode::Char('s') | KeyCode::Char('q') => {
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
                    if let Some(i) = Sources::iter().nth(self.table.state.selected().unwrap_or(0)) {
                        app.src = *i;
                        app.config.source = *i;
                        app.mode = Mode::Loading(LoadType::Searching);
                        match app.config.clone().store() {
                            Ok(_) => app.notify(format!("Updated source to \"{}\"", i.to_string())),
                            Err(e) => app.show_error(format!(
                                "Failed to update default source in config file:\n{}",
                                e
                            )),
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
            ("Esc, Ctrl-s, q", "Close"),
            ("j, ↓", "Down"),
            ("k, ↑", "Up"),
            ("g", "Top"),
            ("G", "Bottom"),
        ])
    }
}
