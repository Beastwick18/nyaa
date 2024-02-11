use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::Constraint,
    style::{Color, Stylize as _},
    text::Text,
    widgets::{Block, Borders, Clear, Row, Table},
    Frame,
};

use crate::app::{App, Mode};

pub struct CatStruct<'a> {
    name: &'a str,
    sub_categories: &'a [&'a str],
    sub_categories_id: &'a [usize],
}

pub static ANIME: CatStruct = CatStruct {
    name: "Anime",
    sub_categories: &[
        "All Anime",
        "English Translated",
        "Non-English Translated",
        "Raw",
        "Anime Music Video",
    ],
    sub_categories_id: &[10, 12, 13, 14, 11],
};
pub static AUDIO: CatStruct = CatStruct {
    name: "Audio",
    sub_categories: &["All Audio", "Audio Lossless", "Audio Lossy"],
    sub_categories_id: &[20, 21, 22],
};

pub static LITERATURE: CatStruct = CatStruct {
    name: "Literature",
    sub_categories: &[
        "All Literature",
        "English-Translated",
        "Non-English Translated",
        "Raw",
    ],
    sub_categories_id: &[30, 31, 32, 33],
};

pub static LIVE_ACTION: CatStruct = CatStruct {
    name: "Live Action",
    sub_categories: &[
        "All Live Action",
        "English-Translated",
        "Idol/Promo Video",
        "Non-English Translated",
        "Raw",
    ],
    sub_categories_id: &[40, 41, 42, 43, 44],
};

pub static PICTURES: CatStruct = CatStruct {
    name: "Pictures",
    sub_categories: &["All Pictures", "Graphics", "Photos"],
    sub_categories_id: &[50, 51, 52],
};

pub static SOFTWARE: CatStruct = CatStruct {
    name: "Software",
    sub_categories: &["All Software", "Applications", "Games"],
    sub_categories_id: &[60, 61, 62],
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
        let area = super::centered_rect(30, 11 as u16 + 2, f.size());
        if let Some(categories) = ALL_CATEGORIES.get(self.major) {
            let mut before: Vec<Row> = vec![];
            let mut after: Vec<Row> = vec![];

            for (i, m) in ALL_CATEGORIES.iter().enumerate() {
                if i < self.major {
                    before.push(Row::new(Text::raw(format!(" ▶ {}", m.name))));
                } else if i == self.major {
                    before.push(Row::new(Text::raw(format!(" ▼ {}", m.name))).bg(Color::DarkGray));
                } else {
                    after.push(Row::new(Text::raw(format!(" ▶ {}", m.name))));
                }
            }
            before.extend(
                categories
                    .sub_categories
                    .iter()
                    .enumerate()
                    .map(|(i, item)| {
                        let id = categories.sub_categories_id[i];
                        let name = if id == self.category {
                            Text::raw(format!("   {}", item))
                        } else {
                            Text::raw(format!("    {}", item))
                        };
                        if i == self.minor {
                            Row::new(name).bg(Color::Blue)
                        } else {
                            Row::new(name)
                        }
                    }),
            );
            before.extend(after);
            let table = Table::new(before, &[Constraint::Percentage(100)])
                .block(Block::new().borders(Borders::ALL).title("Category"));
            f.render_widget(Clear, area);
            f.render_widget(table, area);
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
                        if let Some(id) = cat.sub_categories_id.get(self.minor) {
                            self.category = *id;
                        }
                    }
                    app.mode = Mode::Normal;
                }
                KeyCode::Esc => {
                    app.mode = Mode::Normal;
                }
                KeyCode::Char('j') => {
                    if let Some(cat) = ALL_CATEGORIES.get(self.major) {
                        self.minor = if self.minor + 1 >= cat.sub_categories.len() {
                            0
                        } else {
                            self.minor + 1
                        };
                    }
                }
                KeyCode::Char('k') => {
                    if let Some(cat) = ALL_CATEGORIES.get(self.major) {
                        self.minor = if self.minor < 1 {
                            cat.sub_categories.len() - 1
                        } else {
                            self.minor - 1
                        };
                    }
                }
                KeyCode::Tab => {
                    self.major = if self.major + 1 >= ALL_CATEGORIES.len() {
                        0
                    } else {
                        self.major + 1
                    };
                    self.minor = 0;
                }
                KeyCode::BackTab => {
                    self.major = if self.major < 1 {
                        ALL_CATEGORIES.len() - 1
                    } else {
                        self.major - 1
                    };
                    self.minor = 0;
                }
                _ => {}
            }
        }
    }
}
