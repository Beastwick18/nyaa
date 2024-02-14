use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::Constraint,
    style::{Color, Style, Stylize as _},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Row, Table},
    Frame,
};

use crate::app::{App, Mode};

use super::theme::Theme;

pub struct CatEntry<'a> {
    name: &'a str,
    cfg: &'a str,
    id: usize,
    pub icon: CatIcon,
}

#[derive(Clone)]
pub struct CatIcon {
    pub icon: &'static str,
    pub color: Color,
}

impl<'a> CatEntry<'a> {
    const fn new(
        name: &'a str,
        cfg: &'a str,
        id: usize,
        icon_str: &'static str,
        color: Color,
    ) -> Self {
        let icon = CatIcon {
            icon: icon_str,
            color,
        };
        CatEntry {
            name,
            cfg,
            id,
            icon,
        }
    }
}

pub struct CatStruct<'a> {
    name: &'a str,
    pub entries: &'a [CatEntry<'a>],
}

impl<'a> CatStruct<'a> {
    pub fn find(&'a self, category: u32) -> Option<CatIcon> {
        if let Some(e) = self.entries.iter().find(|e| e.id == category as usize) {
            return Some(e.icon.clone());
        }
        None
    }
}

pub static ANIME: CatStruct = CatStruct {
    name: "Anime",
    entries: &[
        CatEntry::new("All Anime", "AllAnime", 10, "Ani", Color::White),
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
        CatEntry::new("All Audio", "AllAudio", 20, "Aud", Color::White),
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
            "LitEnglishTranslated",
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
                let row = Row::new(vec![Line::from(vec![
                    Span::raw(match e.id == self.category {
                        true => "  ",
                        false => "   ",
                    }),
                    Span::styled(e.icon.icon, Style::new().fg(e.icon.color)),
                    Span::raw(" "),
                    Span::raw(e.name),
                ])]);
                match i == self.minor {
                    true => row.bg(theme.hl_bg),
                    false => row,
                }
            });

            tbl.splice(self.major + 1..self.major + 1, cat_rows);

            let area = super::centered_rect(33, 13, f.size());
            let clear = super::centered_rect(area.width + 2, area.height, f.size());
            f.render_widget(Clear, clear);
            f.render_widget(Block::new().bg(theme.bg), clear);
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
                    app.mode = Mode::Loading;
                }
                KeyCode::Esc | KeyCode::Char('c') | KeyCode::Char('q') => {
                    app.mode = Mode::Normal;
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    if let Some(cat) = ALL_CATEGORIES.get(self.major) {
                        self.minor = match self.minor + 1 >= cat.entries.len() {
                            true => 0,
                            false => self.minor + 1,
                        };
                    }
                }
                KeyCode::Char('k') | KeyCode::Up => {
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
