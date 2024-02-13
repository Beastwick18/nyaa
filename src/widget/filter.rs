use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::Constraint,
    style::{Style, Stylize},
    widgets::{Block, Borders, Row, Table},
};

use crate::app::Mode;

use super::{theme::Theme, EnumIter, Popup, StatefulTable};

#[derive(Clone)]
pub enum Filter {
    NoFilter,
    NoRemakes,
    TrustedOnly,
}

impl EnumIter<Filter> for Filter {
    fn iter() -> std::slice::Iter<'static, Filter> {
        static FILTERS: &'static [Filter] =
            &[Filter::NoFilter, Filter::NoRemakes, Filter::TrustedOnly];
        FILTERS.iter()
    }
}

impl ToString for Filter {
    fn to_string(&self) -> String {
        match self {
            Filter::NoFilter => "No Filter".to_owned(),
            Filter::NoRemakes => "No Remakes".to_owned(),
            Filter::TrustedOnly => "Trusted Only".to_owned(),
        }
    }
}

pub struct FilterPopup {
    pub table: StatefulTable<String>,
    pub selected: Filter,
}

impl Default for FilterPopup {
    fn default() -> Self {
        FilterPopup {
            table: StatefulTable::with_items(Filter::iter().map(|item| item.to_string()).collect()),
            selected: Filter::NoFilter,
        }
    }
}

impl Popup for FilterPopup {
    fn draw(&self, f: &mut ratatui::prelude::Frame, theme: &Theme) {
        let area = super::centered_rect(30, 5, f.size());
        let items = self.table.items.iter().enumerate().map(|(i, item)| {
            match i == (self.selected.to_owned() as usize) {
                true => Row::new(vec![format!("  {}", item.to_owned())]),
                false => Row::new(vec![format!("   {}", item.to_owned())]),
            }
        });
        let table = Table::new(items, [Constraint::Percentage(100)])
            .block(
                Block::new()
                    .border_style(Style::new().fg(theme.border_focused_color))
                    .borders(Borders::ALL)
                    .border_type(theme.border)
                    .title("Filter"),
            )
            .fg(theme.fg)
            .bg(theme.bg)
            .highlight_style(Style::default().bg(theme.hl_bg).fg(theme.hl_fg));
        f.render_stateful_widget(table, area, &mut self.table.state.to_owned());
    }

    fn handle_event(&mut self, app: &mut crate::app::App, e: &crossterm::event::Event) {
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
                KeyCode::Char('j') => {
                    self.table.next_wrap(1);
                }
                KeyCode::Char('k') => {
                    self.table.next_wrap(-1);
                }
                KeyCode::Char('G') => {
                    self.table.select(self.table.items.len() - 1);
                }
                KeyCode::Char('g') => {
                    self.table.select(0);
                }
                KeyCode::Enter => {
                    if let Some(i) =
                        Filter::iter().nth(self.table.state.selected().unwrap_or_default())
                    {
                        self.selected = i.to_owned();
                        app.mode = Mode::Normal;
                    }
                }
                _ => {}
            }
        }
    }
}
