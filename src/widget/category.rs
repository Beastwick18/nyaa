use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::Constraint,
    style::{Style, Stylize as _},
    text::Text,
    widgets::{Block, Borders, Clear, Row, Table},
    Frame,
};

use crate::app::{App, Mode};

use super::theme::Theme;

pub struct CatEntry<'a> {
    name: &'a str,
    cfg: &'a str,
    id: usize,
}

impl<'a> CatEntry<'a> {
    const fn new(name: &'a str, cfg: &'a str, id: usize) -> Self {
        CatEntry { name, cfg, id }
    }
}

pub struct CatStruct<'a> {
    name: &'a str,
    entries: &'a [CatEntry<'a>],
}

pub static ANIME: CatStruct = CatStruct {
    name: "Anime",
    entries: &[
        CatEntry::new("All Anime", "AllAnime", 10),
        CatEntry::new("English Translated", "AnimeEnglishTranslated", 12),
        CatEntry::new("Non-English Translated", "AnimeNonEnglishTranslated", 13),
        CatEntry::new("Raw", "AnimeRaw", 14),
        CatEntry::new("Anime Music Video", "AnimeMusicVideo", 11),
    ],
};

pub static AUDIO: CatStruct = CatStruct {
    name: "Audio",
    entries: &[
        CatEntry::new("All Audio", "AllAudio", 20),
        CatEntry::new("Lossless", "AudioLossless", 21),
        CatEntry::new("Lossy", "AudioLossy", 22),
    ],
};

pub static LITERATURE: CatStruct = CatStruct {
    name: "Literature",
    entries: &[
        CatEntry::new("All Literature", "AllLiterature", 30),
        CatEntry::new("English-Translated", "LitEnglishTranslated", 31),
        CatEntry::new("Non-English Translated", "LitEnglishTranslated", 32),
        CatEntry::new("Raw", "LitRaw", 33),
    ],
};

pub static LIVE_ACTION: CatStruct = CatStruct {
    name: "Live Action",
    entries: &[
        CatEntry::new("All Live Action", "AllLiveAction", 40),
        CatEntry::new("English-Translated", "LiveEnglishTranslated", 41),
        CatEntry::new("Non-English Translated", "LiveNonEnglishTranslated", 43),
        CatEntry::new("Idol/Promo Video", "LiveIdolPromoVideo", 42),
        CatEntry::new("Raw", "LiveRaw", 44),
    ],
};

pub static PICTURES: CatStruct = CatStruct {
    name: "Pictures",
    entries: &[
        CatEntry::new("All Pictures", "AllPictures", 50),
        CatEntry::new("Graphics", "PicGraphics", 51),
        CatEntry::new("Photos", "PicPhotos", 52),
    ],
};

pub static SOFTWARE: CatStruct = CatStruct {
    name: "Software",
    entries: &[
        CatEntry::new("All Software", "AllSoftware", 60),
        CatEntry::new("Applications", "SoftApplications", 61),
        CatEntry::new("Games", "SoftGames", 62),
    ],
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
    fn draw(&self, f: &mut Frame, theme: &Theme) {
        if let Some(cat) = ALL_CATEGORIES.get(self.major) {
            let area = super::centered_rect(30, 13, f.size());
            let mut tbl: Vec<Row> = ALL_CATEGORIES
                .iter()
                .enumerate()
                .map(|(i, e)| match i == self.major {
                    false => Row::new(Text::raw(format!(" ▶ {}", e.name))),
                    true => Row::new(Text::raw(format!(" ▼ {}", e.name)))
                        .bg(theme.solid_bg)
                        .fg(theme.solid_fg),
                })
                .collect();

            let cat_rows = cat.entries.iter().enumerate().map(|(i, e)| {
                let row = Row::new(Text::raw(match e.id == self.category {
                    true => format!("   {}", e.name),
                    false => format!("    {}", e.name),
                }));
                match i == self.minor {
                    true => row.bg(theme.hl_bg),
                    false => row,
                }
            });

            tbl.splice(self.major + 1..self.major + 1, cat_rows);
            f.render_widget(Clear, area);
            f.render_widget(
                Table::new(tbl, &[Constraint::Percentage(100)])
                    .block(
                        Block::new()
                            .border_style(Style::new().fg(theme.border_focused_color))
                            .border_type(theme.border)
                            .borders(Borders::ALL)
                            .title("Category"),
                    )
                    .fg(theme.fg)
                    .bg(theme.bg),
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
                        if let Some(item) = cat.entries.get(self.minor) {
                            self.category = item.id;
                        }
                    }
                    app.mode = Mode::Normal;
                }
                KeyCode::Esc | KeyCode::Char('c') | KeyCode::Char('q') => {
                    app.mode = Mode::Normal;
                }
                KeyCode::Char('j') => {
                    if let Some(cat) = ALL_CATEGORIES.get(self.major) {
                        self.minor = match self.minor + 1 >= cat.entries.len() {
                            true => 0,
                            false => self.minor + 1,
                        };
                    }
                }
                KeyCode::Char('k') => {
                    if let Some(cat) = ALL_CATEGORIES.get(self.major) {
                        self.minor = match self.minor < 1 {
                            true => cat.entries.len() - 1,
                            false => self.minor - 1,
                        };
                    }
                }
                KeyCode::Tab | KeyCode::Char('J') => {
                    self.major = match self.major + 1 >= ALL_CATEGORIES.len() {
                        true => 0,
                        false => self.major + 1,
                    };
                    self.minor = 0;
                }
                KeyCode::BackTab | KeyCode::Char('K') => {
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
