use std::cmp::{max, min};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Alignment, Constraint, Margin, Rect},
    text::Line,
    widgets::{Row, Scrollbar, ScrollbarOrientation, StatefulWidget as _, Table},
    Frame,
};

use crate::{
    app::{App, Mode},
    style,
};

use super::{border_block, StatefulTable, Widget};

pub struct HelpPopup {
    pub table: StatefulTable<(&'static str, &'static str)>,
    pub prev_mode: Mode,
}

impl Default for HelpPopup {
    fn default() -> Self {
        HelpPopup {
            table: StatefulTable::with_items(vec![]),
            prev_mode: Mode::Normal,
        }
    }
}

impl HelpPopup {
    pub fn with_items(&mut self, items: Vec<(&'static str, &'static str)>, prev_mode: Mode) {
        self.table.scrollbar_state = self.table.scrollbar_state.content_length(items.len());
        self.table.items = items;
        self.prev_mode = prev_mode;
    }
}

impl Widget for HelpPopup {
    fn draw(&mut self, f: &mut Frame, app: &App, area: Rect) {
        let buf = f.buffer_mut();
        let iter = self.table.items.iter();

        // Get max len of Key
        let key_min = iter.clone().fold(15, |acc, e| max(acc, e.0.len())) as u16;
        // Get max len of action
        let map_min = iter.fold(15, |acc, e| max(acc, e.1.len())) as u16;
        // Cap height between the number of entries + 3 for padding, and 20
        let height = min(25, self.table.items.len() + 3) as u16;

        let center = super::centered_rect(key_min + map_min + 6, height, area);
        let clear = super::centered_rect(center.width + 2, center.height, area);
        let items = self.table.items.iter().map(|(key, map)| {
            Row::new([
                Line::from(key.to_string()).alignment(Alignment::Right),
                Line::from("⇒"),
                Line::from(map.to_string()),
            ])
        });
        let header = Row::new([
            Line::from("Key").alignment(Alignment::Center),
            Line::from(""),
            Line::from("Action").alignment(Alignment::Center),
        ])
        .style(style!(bold, underlined, fg:app.theme.border_focused_color))
        .height(1)
        .bottom_margin(0);
        let table = Table::new(items, [Constraint::Percentage(100)])
            .block(
                border_block(app.theme, true)
                    .title(format!("Help: {}", self.prev_mode.to_string())),
            )
            .header(header)
            .widths(Constraint::from_lengths([key_min, 1, map_min]))
            .highlight_style(style!(bg:app.theme.hl_bg));

        super::clear(clear, buf, app.theme.bg);
        table.render(center, buf, &mut self.table.state);

        // Only show scrollbar if content overflows
        if self.table.items.len() as u16 + 2 >= center.height {
            let sb = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .track_symbol(Some("│"))
                .begin_symbol(Some(""))
                .end_symbol(None);
            let sb_area = center.inner(&Margin {
                vertical: 1,
                horizontal: 0,
            });
            sb.render(sb_area, buf, &mut self.table.scrollbar_state);
        }
    }

    fn handle_event(&mut self, app: &mut crate::app::App, e: &crossterm::event::Event) {
        if let Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            ..
        }) = e
        {
            match code {
                KeyCode::Esc | KeyCode::Char('?') | KeyCode::F(1) | KeyCode::Char('q') => {
                    app.mode = self.prev_mode.to_owned();
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
                _ => {}
            }
        }
    }

    fn get_help() -> Option<Vec<(&'static str, &'static str)>> {
        None
    }
}
