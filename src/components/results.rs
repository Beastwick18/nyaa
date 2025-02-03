use color_eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Alignment, Constraint, Rect},
    style::{Color, Style, Stylize as _},
    symbols::line,
    text::Line,
    widgets::{Block, Borders, StatefulWidget, Table, TableState},
    Frame,
};

use crate::{
    action::AppAction,
    app::Context,
    keys::{self, KeyCombo},
    result::{ResultCell, ResultHeader, ResultTable},
};

use super::Component;

pub struct ResultsComponent {
    results: ResultTable,
    current_keycombo: String,
    current_keycombo_color: Color,
}

impl ResultsComponent {
    pub fn new() -> Self {
        let header: &mut [ResultHeader] = &mut [
            ("Cat", None).into(),
            ("Name", None).into(),
            (ResultCell::new("Size").alignment(Alignment::Center), None).into(),
            (
                ResultCell::new("Date").alignment(Alignment::Center),
                Some('▼'),
            )
                .into(),
            (ResultCell::new("").alignment(Alignment::Center), None).into(),
            (ResultCell::new("").alignment(Alignment::Center), None).into(),
            (ResultCell::new("").alignment(Alignment::Center), None).into(),
        ];
        Self {
            results: ResultTable::new([[
                ResultCell::new("Raw").fg(Color::Green),
                ResultCell::new("恋するMOON DOG raw 第13巻").fg(Color::White),
                ResultCell::new("54.5 KiB").fg(Color::DarkGray),
                ResultCell::new("2025-02-02 22:18").fg(Color::White),
                ResultCell::new("12").fg(Color::Green),
                ResultCell::new("2").fg(Color::Red),
                ResultCell::new("24").fg(Color::White),
            ]])
            .header(header.to_vec())
            .header_style(Style::new().underlined())
            .apply_alignment([
                Alignment::Center,
                Alignment::Left,
                Alignment::Right,
                Alignment::Center,
                Alignment::Right,
                Alignment::Right,
                Alignment::Left,
            ])
            .binding([
                3.into(),
                Constraint::Fill(5),
                12.into(),
                16.into(),
                3.into(),
                3.into(),
                3.into(),
            ]),
            current_keycombo: String::new(),
            current_keycombo_color: Color::White,
        }
    }
}

impl Component for ResultsComponent {
    fn update(
        &mut self,
        ctx: &Context,
        _action: &AppAction,
    ) -> color_eyre::eyre::Result<Option<AppAction>> {
        let (keycombo, keycombo_color) = if ctx.keycombo.is_empty() && ctx.last_keycombo.is_some() {
            match ctx.last_keycombo.as_ref().unwrap() {
                KeyCombo::Successful(vec) => (vec, Color::Cyan),
                KeyCombo::Cancelled(vec) => (vec, Color::DarkGray),
                KeyCombo::Unmatched(vec) => (vec, Color::Red),
            }
        } else {
            (&ctx.keycombo, Color::White)
        };
        self.current_keycombo = keycombo.iter().map(keys::key_event_to_string).collect();
        self.current_keycombo_color = keycombo_color;

        Ok(None)
    }

    fn on_key(&mut self, _ctx: &Context, _key: &KeyEvent) -> Result<()> {
        // if ctx.mode != Mode::Home {
        //     return Ok(());
        // }
        Ok(())
    }

    fn render(&mut self, _ctx: &Context, frame: &mut Frame, area: Rect) -> Result<()> {
        let mut block = Block::new()
            .fg(Color::Rgb(255, 255, 255))
            .borders(Borders::ALL);
        if !self.current_keycombo.is_empty() {
            let combo = self
                .current_keycombo
                .clone()
                .fg(self.current_keycombo_color);
            let vr = line::NORMAL.vertical_right;
            let vl = line::NORMAL.vertical_left;
            let keycombo = Line::from_iter([
                format!("{} ", vl).fg(Color::Rgb(255, 255, 255)),
                combo,
                format!(" {}", vr).fg(Color::Rgb(255, 255, 255)),
            ]);

            block = block.title_bottom(keycombo.right_aligned());
        };

        let table: Table = self.results.clone().into();
        table
            .block(block)
            .render(area, frame.buffer_mut(), &mut TableState::default());
        Ok(())
    }
}
