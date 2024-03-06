use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Stylize as _},
    text::{Line, Span, Text},
    widgets::{Row, Table, Widget as _},
    Frame,
};

use crate::app::{App, LoadType, Mode};

use super::{border_block, Widget};

pub struct CatEntry {
    name: &'static str,
    pub cfg: &'static str,
    pub id: usize,
    pub icon: CatIcon,
}

#[derive(Clone)]
pub struct CatIcon {
    pub label: &'static str,
    pub color: Color,
}

impl Default for CatIcon {
    fn default() -> Self {
        CatIcon {
            label: "???",
            color: Color::Gray,
        }
    }
}

impl CatEntry {
    pub fn from_str(s: &str) -> &Self {
        let split: Vec<&str> = s.split('_').collect();
        let high = split.first().unwrap_or(&"1").parse().unwrap_or(1);
        let low = split.last().unwrap_or(&"0").parse().unwrap_or(0);
        let id = high * 10 + low;
        for cat in ALL_CATEGORIES {
            if let Some(ent) = cat.entries.iter().find(|ent| ent.id == id) {
                return ent;
            }
        }
        &ALL_CATEGORIES[0].entries[0]
    }
}

impl CatEntry {
    const fn new(
        name: &'static str,
        cfg: &'static str,
        id: usize,
        label: &'static str,
        color: Color,
    ) -> Self {
        CatEntry {
            name,
            cfg,
            id,
            icon: CatIcon { label, color },
        }
    }
}

pub struct CatStruct {
    name: &'static str,
    pub entries: &'static [CatEntry],
}

pub static ALL: CatStruct = CatStruct {
    name: "All Categories",
    entries: &[CatEntry::new(
        "All Categories",
        "AllCategories",
        0,
        "---",
        Color::White,
    )],
};

pub static ANIME: CatStruct = CatStruct {
    name: "Anime",
    entries: &[
        CatEntry::new("All Anime", "AllAnime", 10, "Ani", Color::Gray),
        CatEntry::new(
            "English Translated",
            "AnimeEnglishTranslated",
            12,
            "Sub",
            Color::LightMagenta,
        ),
        CatEntry::new(
            "Non-English Translated",
            "AnimeNonEnglishTranslated",
            13,
            "Sub",
            Color::LightGreen,
        ),
        CatEntry::new("Raw", "AnimeRaw", 14, "Raw", Color::Gray),
        CatEntry::new(
            "Anime Music Video",
            "AnimeMusicVideo",
            11,
            "AMV",
            Color::Magenta,
        ),
    ],
};

pub static AUDIO: CatStruct = CatStruct {
    name: "Audio",
    entries: &[
        CatEntry::new("All Audio", "AllAudio", 20, "Aud", Color::Gray),
        CatEntry::new("Lossless", "AudioLossless", 21, "Aud", Color::Red),
        CatEntry::new("Lossy", "AudioLossy", 22, "Aud", Color::Yellow),
    ],
};

pub static LITERATURE: CatStruct = CatStruct {
    name: "Literature",
    entries: &[
        CatEntry::new("All Literature", "AllLiterature", 30, "Lit", Color::Gray),
        CatEntry::new(
            "English-Translated",
            "LitEnglishTranslated",
            31,
            "Lit",
            Color::LightGreen,
        ),
        CatEntry::new(
            "Non-English Translated",
            "LitNonEnglishTranslated",
            32,
            "Lit",
            Color::Yellow,
        ),
        CatEntry::new("Raw", "LitRaw", 33, "Lit", Color::Green),
    ],
};

pub static LIVE_ACTION: CatStruct = CatStruct {
    name: "Live Action",
    entries: &[
        CatEntry::new("All Live Action", "AllLiveAction", 40, "Liv", Color::Gray),
        CatEntry::new(
            "English-Translated",
            "LiveEnglishTranslated",
            41,
            "Liv",
            Color::Yellow,
        ),
        CatEntry::new(
            "Non-English Translated",
            "LiveNonEnglishTranslated",
            43,
            "Liv",
            Color::LightCyan,
        ),
        CatEntry::new(
            "Idol/Promo Video",
            "LiveIdolPromoVideo",
            42,
            "Liv",
            Color::LightYellow,
        ),
        CatEntry::new("Raw", "LiveRaw", 44, "Liv", Color::Gray),
    ],
};

pub static PICTURES: CatStruct = CatStruct {
    name: "Pictures",
    entries: &[
        CatEntry::new("All Pictures", "AllPictures", 50, "Pic", Color::Gray),
        CatEntry::new("Graphics", "PicGraphics", 51, "Pic", Color::LightMagenta),
        CatEntry::new("Photos", "PicPhotos", 52, "Pic", Color::Magenta),
    ],
};

pub static SOFTWARE: CatStruct = CatStruct {
    name: "Software",
    entries: &[
        CatEntry::new("All Software", "AllSoftware", 60, "Sof", Color::Gray),
        CatEntry::new("Applications", "SoftApplications", 61, "Sof", Color::Blue),
        CatEntry::new("Games", "SoftGames", 62, "Sof", Color::LightBlue),
    ],
};

pub static ALL_CATEGORIES: &[&CatStruct] = &[
    &ALL,
    &ANIME,
    &AUDIO,
    &LITERATURE,
    &LIVE_ACTION,
    &PICTURES,
    &SOFTWARE,
];

pub fn find_category<S: Into<String>>(name: S) -> Option<&'static CatEntry> {
    let name = name.into();
    for cat in ALL_CATEGORIES {
        if let Some(ent) = cat
            .entries
            .iter()
            .find(|ent| ent.cfg.eq_ignore_ascii_case(&name))
        {
            return Some(ent);
        }
    }
    None
}

#[derive(Default)]
pub struct CategoryPopup {
    pub category: usize,
    pub major: usize,
    pub minor: usize,
}

impl CategoryPopup {
    fn next_tab(&mut self) {
        self.major = match self.major + 1 >= ALL_CATEGORIES.len() {
            true => 0,
            false => self.major + 1,
        };
        self.minor = 0;
    }

    fn prev_tab(&mut self) {
        self.major = match self.major == 0 {
            true => ALL_CATEGORIES.len() - 1,
            false => self.major - 1,
        };
        self.minor = 0;
    }
}

impl Widget for CategoryPopup {
    fn draw(&self, f: &mut Frame, app: &App, area: Rect) {
        if let Some(cat) = ALL_CATEGORIES.get(self.major) {
            let mut tbl: Vec<Row> = ALL_CATEGORIES
                .iter()
                .enumerate()
                .map(|(i, e)| match i == self.major {
                    false => Row::new(Text::raw(format!(" ▶ {}", e.name))),
                    true => Row::new(Text::raw(format!(" ▼ {}", e.name)))
                        .bg(app.theme.solid_bg)
                        .fg(app.theme.solid_fg),
                })
                .collect();

            let cat_rows = cat.entries.iter().enumerate().map(|(i, e)| {
                let row = Row::new(vec![Line::from(vec![
                    Span::raw(match e.id == self.category {
                        true => "  ",
                        false => "   ",
                    }),
                    e.icon.label.fg(e.icon.color),
                    Span::raw(" "),
                    Span::raw(e.name),
                ])]);
                match i == self.minor {
                    true => row.bg(app.theme.hl_bg),
                    false => row,
                }
            });

            tbl.splice(self.major + 1..self.major + 1, cat_rows);

            let center = super::centered_rect(33, 14, area);
            let clear = super::centered_rect(center.width + 2, center.height, area);
            super::clear(clear, f.buffer_mut(), app.theme.bg);
            Table::new(tbl, [Constraint::Percentage(100)])
                .block(border_block(app.theme, true).title("Category"))
                .render(center, f.buffer_mut());
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
                        if let Some(item) = cat.entries.get(self.minor) {
                            self.category = item.id;
                        }
                    }
                    app.mode = Mode::Loading(LoadType::Categorizing);
                }
                KeyCode::Esc | KeyCode::Char('c') | KeyCode::Char('q') => {
                    app.mode = Mode::Normal;
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    if let Some(cat) = ALL_CATEGORIES.get(self.major) {
                        self.minor = match self.minor + 1 >= cat.entries.len() {
                            true => {
                                self.next_tab();
                                0
                            }
                            false => self.minor + 1,
                        };
                    }
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    if ALL_CATEGORIES.get(self.major).is_some() {
                        self.minor = match self.minor < 1 {
                            true => {
                                self.prev_tab();
                                match ALL_CATEGORIES.get(self.major) {
                                    Some(cat) => cat.entries.len() - 1,
                                    None => 0,
                                }
                            }
                            false => self.minor - 1,
                        };
                    }
                }
                KeyCode::Char('G') => {
                    if let Some(cat) = ALL_CATEGORIES.get(self.major) {
                        self.minor = cat.entries.len() - 1;
                    }
                }
                KeyCode::Char('g') => {
                    self.minor = 0;
                }
                KeyCode::Tab | KeyCode::Char('J') => {
                    self.next_tab();
                }
                KeyCode::BackTab | KeyCode::Char('K') => {
                    self.prev_tab();
                }
                _ => {}
            }
        }
    }

    fn get_help() -> Option<Vec<(&'static str, &'static str)>> {
        Some(vec![
            ("Enter", "Confirm"),
            ("Esc, c, q", "Close"),
            ("j, ↓", "Down"),
            ("k, ↑", "Up"),
            ("g", "Top"),
            ("G", "Bottom"),
            ("Tab, J", "Next Tab"),
            ("S-Tab, K", "Prev Tab"),
        ])
    }
}
