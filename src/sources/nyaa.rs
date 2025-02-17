use ratatui::{
    layout::{Alignment, Constraint},
    style::{Color, Style, Stylize},
};

use crate::result::{ResultCell, ResultHeaderCell, ResultTable, Results};

use super::SourceTask;

pub struct NyaaSource;

impl SourceTask for NyaaSource {
    fn search(&self, query: String) -> Results {
        let items = Vec::new();

        let header: [ResultHeaderCell; 7] = [
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

        let rows = [
            [
                ResultCell::new("Raw").fg(Color::Green),
                ResultCell::new("恋するMOON DOG raw 第13巻").fg(Color::White),
                ResultCell::new("54.5 KiB").fg(Color::DarkGray),
                ResultCell::new("2025-02-02 22:18").fg(Color::White),
                ResultCell::new("12").fg(Color::Green),
                ResultCell::new("2").fg(Color::Red),
                ResultCell::new("24").fg(Color::White),
            ],
            [
                ResultCell::new("Qry").fg(Color::Green),
                ResultCell::new(query).fg(Color::White),
                ResultCell::new("12.2 KiB").fg(Color::DarkGray),
                ResultCell::new("2025-02-02 22:18").fg(Color::White),
                ResultCell::new("12").fg(Color::Green),
                ResultCell::new("2").fg(Color::Red),
                ResultCell::new("24").fg(Color::White),
            ],
        ];

        let alignment = [
            Alignment::Center,
            Alignment::Left,
            Alignment::Right,
            Alignment::Center,
            Alignment::Left,
            Alignment::Left,
            Alignment::Left,
        ];

        let binding = [
            3.into(),
            Constraint::Fill(5),
            12.into(),
            16.into(),
            3.into(),
            3.into(),
            3.into(),
        ];

        let table = ResultTable::new(rows)
            .header(header)
            .header_style(Style::new().underlined())
            .apply_alignment(alignment)
            .binding(binding);

        Results { items, table }
    }
}
