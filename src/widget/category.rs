use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::Constraint,
    style::{Color, Stylize as _},
    text::Text,
    widgets::{Row, Table},
    Frame,
};

use crate::{
    app::{App, Mode},
    ui,
};

pub struct CatStruct<'a> {
    name: &'a str,
    sub: &'a [&'a str],
    sub_id: &'a [usize],
}

pub static ANIME: CatStruct = CatStruct {
    name: "Anime",
    sub: &[
        "All Anime",
        "English Translated",
        "Non-English Translated",
        "Raw",
        "Anime Music Video",
    ],
    sub_id: &[10, 12, 13, 14, 11],
};
pub static AUDIO: CatStruct = CatStruct {
    name: "Audio",
    sub: &["All Audio", "Audio Lossless", "Audio Lossy"],
    sub_id: &[20, 21, 22],
};

pub static LITERATURE: CatStruct = CatStruct {
    name: "Literature",
    sub: &[
        "All Literature",
        "English-Translated",
        "Non-English Translated",
        "Raw",
    ],
    sub_id: &[30, 31, 32, 33],
};

pub static LIVE_ACTION: CatStruct = CatStruct {
    name: "Live Action",
    sub: &[
        "All Live Action",
        "English-Translated",
        "Idol/Promo Video",
        "Non-English Translated",
        "Raw",
    ],
    sub_id: &[40, 41, 42, 43, 44],
};

pub static PICTURES: CatStruct = CatStruct {
    name: "Pictures",
    sub: &["All Pictures", "Graphics", "Photos"],
    sub_id: &[50, 51, 52],
};

pub static SOFTWARE: CatStruct = CatStruct {
    name: "Software",
    sub: &["All Software", "Applications", "Games"],
    sub_id: &[60, 61, 62],
};

pub static ALL_CATEGORIES: &'static [&CatStruct] = &[
    &ANIME,
    &AUDIO,
    &LITERATURE,
    &LIVE_ACTION,
    &PICTURES,
    &SOFTWARE,
];

pub struct CategoryPopup {
    pub category: usize,
    pub major: usize,
    pub minor: usize,
}

impl Default for CategoryPopup {
    fn default() -> Self {
        return CategoryPopup {
            category: 10,
            major: 0,
            minor: 0,
        };
    }
}

impl super::Popup for CategoryPopup {
    fn draw(&self, f: &mut Frame) {
        if let Some(cat) = ALL_CATEGORIES.get(self.major) {
            let area = super::centered_rect(30, 13, f.size());
            let mut tbl: Vec<Row> = ALL_CATEGORIES
                .iter()
                .enumerate()
                .map(|(i, e)| match i == self.major {
                    false => Row::new(Text::raw(format!(" ▶ {}", e.name))),
                    true => Row::new(Text::raw(format!(" ▼ {}", ALL_CATEGORIES[self.major].name)))
                        .bg(Color::DarkGray),
                })
                .collect();

            let cat_rows = cat.sub.iter().enumerate().map(|(i, item)| {
                let row = Row::new(Text::raw(match cat.sub_id[i] == self.category {
                    true => format!("   {}", item),
                    false => format!("    {}", item),
                }));
                match i == self.minor {
                    true => row.bg(Color::Blue),
                    false => row,
                }
            });

            tbl.splice(self.major + 1..self.major + 1, cat_rows);
            f.render_widget(
                Table::new(tbl, &[Constraint::Percentage(100)]).block(ui::HI_BLOCK.to_owned()),
                area,
            );
        }
    }

    fn handle_event(&mut self, app: &mut App, e: &Event) {
        if let Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            ..
        }) = e
        {
            match code {
                KeyCode::Enter => {
                    if let Some(cat) = ALL_CATEGORIES.get(self.major) {
                        if let Some(id) = cat.sub_id.get(self.minor) {
                            self.category = *id;
                        }
                    }
                    app.mode = Mode::Normal;
                }
                KeyCode::Esc | KeyCode::Char('c') | KeyCode::Char('q') => {
                    app.mode = Mode::Normal;
                }
                KeyCode::Char('j') => {
                    if let Some(cat) = ALL_CATEGORIES.get(self.major) {
                        self.minor = match self.minor + 1 >= cat.sub.len() {
                            true => 0,
                            false => self.minor + 1,
                        };
                    }
                }
                KeyCode::Char('k') => {
                    if let Some(cat) = ALL_CATEGORIES.get(self.major) {
                        self.minor = match self.minor < 1 {
                            true => cat.sub.len() - 1,
                            false => self.minor - 1,
                        };
                    }
                }
                KeyCode::Tab => {
                    self.major = match self.major + 1 >= ALL_CATEGORIES.len() {
                        true => 0,
                        false => self.major + 1,
                    };
                    self.minor = 0;
                }
                KeyCode::BackTab => {
                    self.major = match self.major < 1 {
                        true => ALL_CATEGORIES.len() - 1,
                        false => self.major - 1,
                    };
                    self.minor = 0;
                }
                _ => {}
            }
        }
    }
}
