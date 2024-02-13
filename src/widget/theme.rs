use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::Constraint,
    style::{Color, Style, Stylize},
    widgets::{Block, BorderType, Borders, Row, Table},
};

use crate::app::Mode;

use super::{Popup, StatefulTable};

pub struct Theme {
    pub name: &'static str,
    pub bg: Color,
    pub fg: Color,
    pub border: BorderType,
    pub border_color: Color,
    pub border_focused_color: Color,
    pub hl_bg: Color,
    pub hl_fg: Color,
    pub solid_bg: Color,
    pub solid_fg: Color,
}

pub static THEMES: &'static [&'static Theme] = &[
    &Theme {
        name: "Default",
        bg: Color::Black,
        fg: Color::Green,
        border: BorderType::Plain,
        border_color: Color::White,
        border_focused_color: Color::LightCyan,
        hl_bg: Color::Rgb(60, 60, 60),
        hl_fg: Color::White,
        solid_bg: Color::White,
        solid_fg: Color::Black,
    },
    &Theme {
        name: "Dracula",
        bg: Color::Rgb(40, 42, 54),
        fg: Color::Rgb(248, 248, 242),
        border: BorderType::Rounded,
        border_color: Color::Rgb(98, 114, 164),
        border_focused_color: Color::Rgb(189, 147, 249),
        hl_bg: Color::Rgb(68, 71, 90),
        hl_fg: Color::Rgb(248, 248, 242),
        solid_fg: Color::Rgb(40, 42, 54),
        solid_bg: Color::Rgb(139, 233, 253),
    },
    &Theme {
        name: "Gruvbox",
        bg: Color::Rgb(40, 40, 40),
        fg: Color::Rgb(235, 219, 178),
        border: BorderType::Double,
        border_color: Color::Rgb(80, 73, 69),
        border_focused_color: Color::Rgb(214, 93, 14),
        hl_bg: Color::Rgb(146, 131, 116),
        hl_fg: Color::Rgb(29, 32, 33),
        solid_bg: Color::Rgb(69, 133, 136),
        solid_fg: Color::Rgb(235, 219, 178),
    },
];

pub struct ThemePopup {
    pub table: StatefulTable<String>,
    pub selected: usize,
}

impl Default for ThemePopup {
    fn default() -> Self {
        ThemePopup {
            table: StatefulTable::with_items(
                THEMES.iter().map(|item| item.name.to_owned()).collect(),
            ),
            selected: 0,
        }
    }
}

impl Popup for ThemePopup {
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
                    .title("Theme"),
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
                    if let Some(i) = THEMES
                        .iter()
                        .nth(self.table.state.selected().unwrap_or_default())
                    {
                        self.selected = self.table.state.selected().unwrap_or_default();
                        app.theme = i;
                        app.mode = Mode::Normal;
                    }
                }
                _ => {}
            }
        }
    }
}
