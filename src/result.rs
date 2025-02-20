use std::borrow::Borrow;

use derive_more::{Deref, DerefMut};
use ratatui::{
    layout::{Alignment, Constraint},
    style::{Style, Stylize},
    text::Text,
    widgets::{Cell, Row, Table},
};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResultItem {
    id: String,                   // Unique ID
    name: String,                 // Name of torrent on website
    magnet_link: Option<String>,  // Torrents magnet link
    torrent_link: Option<String>, // Link to torrent file
    post_link: Option<String>,    // Link to forum post
    filename: String,             // Filename on website
    seeders: u16,                 // Number of seeders
    leechers: u16,                // Number of leechers
    downloads: u16,               // Total downloads
    size: usize,                  // Size of file in bytes
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Results {
    pub items: Vec<ResultItem>,
    pub table: ResultTable,
}

#[derive(Clone, Default, Deref, DerefMut)]
pub struct ResultHeaderCell {
    #[deref]
    #[deref_mut]
    cell: ResultCell,
    status: Option<char>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResultCell {
    content: String,
    style: Style,
    alignment: Alignment,
}

impl ResultCell {
    pub fn new<S>(content: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            content: content.into(),
            ..Default::default()
        }
    }

    pub fn style<S>(mut self, style: S) -> Self
    where
        S: Into<Style>,
    {
        self.style = style.into();
        self
    }

    pub fn alignment<A>(mut self, alignment: A) -> Self
    where
        A: Into<Alignment>,
    {
        self.alignment = alignment.into();
        self
    }

    pub fn set_alignment<A>(&mut self, alignment: A)
    where
        A: Into<Alignment>,
    {
        self.alignment = alignment.into();
    }
}

impl<'a> Stylize<'a, ResultCell> for ResultCell {
    fn bg<C: Into<ratatui::prelude::Color>>(mut self, color: C) -> Self {
        self.style = self.style.bg(color.into());
        self
    }

    fn fg<C: Into<ratatui::prelude::Color>>(mut self, color: C) -> Self {
        self.style = self.style.fg(color.into());
        self
    }

    fn reset(mut self) -> Self {
        self.style = self.style.reset();
        self
    }

    fn add_modifier(mut self, modifier: ratatui::prelude::Modifier) -> Self {
        self.style = self.style.add_modifier(modifier);
        self
    }

    fn remove_modifier(mut self, modifier: ratatui::prelude::Modifier) -> Self {
        self.style = self.style.remove_modifier(modifier);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResultRow {
    cells: Vec<ResultCell>,
    style: Style,
}

impl ResultRow {
    pub fn new<C>(cells: C) -> Self
    where
        C: IntoIterator,
        C::Item: Into<ResultCell>,
    {
        Self {
            cells: cells.into_iter().map(Into::into).collect(),
            ..Default::default()
        }
    }

    pub fn style<S>(mut self, style: S) -> Self
    where
        S: Into<Style>,
    {
        self.style = style.into();
        self
    }

    pub fn alignment<A>(mut self, alignment: A) -> Self
    where
        A: IntoIterator,
        A::Item: Into<Alignment>,
    {
        let a: Vec<Alignment> = alignment.into_iter().map(Into::into).collect();

        for (align, cell) in a.iter().zip(self.cells.iter_mut()) {
            cell.alignment = *align;
        }
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResultTable {
    header: ResultRow,
    binding: Vec<Constraint>,
    rows: Vec<ResultRow>,
}

impl ResultTable {
    pub fn new<R>(rows: R) -> Self
    where
        R: IntoIterator,
        R::Item: Into<ResultRow>,
    {
        Self {
            rows: rows.into_iter().map(Into::into).collect(),
            ..Default::default()
        }
    }

    pub fn header<R>(mut self, header: R) -> Self
    where
        R: IntoIterator,
        R::Item: Into<ResultHeaderCell>,
    {
        self.header = header
            .into_iter()
            .map(Into::<ResultHeaderCell>::into)
            .map(Into::<ResultCell>::into)
            .collect::<Vec<ResultCell>>()
            .into();
        self
    }

    pub fn header_style(mut self, style: Style) -> Self {
        self.header.style = style;
        self
    }

    pub fn binding<C>(mut self, binding: C) -> Self
    where
        C: IntoIterator,
        C::Item: Into<Constraint>,
    {
        self.binding = binding.into_iter().map(Into::into).collect();
        self
    }

    pub fn apply_alignment<A>(mut self, alignment: A) -> Self
    where
        A: IntoIterator,
        A::Item: Into<Alignment>,
    {
        let a: Vec<Alignment> = alignment.into_iter().map(Into::into).collect();

        for row in self.rows.iter_mut() {
            for (align, cell) in a.iter().zip(row.cells.iter_mut()) {
                cell.alignment = *align;
            }
        }
        self
    }
}

impl<'a> From<&'a ResultTable> for Table<'a> {
    fn from(res: &'a ResultTable) -> Self {
        Table::new(&res.rows, &res.binding).header((&res.header).into())
    }
}

impl<'a> From<&'a ResultRow> for Row<'a> {
    fn from(rrow: &'a ResultRow) -> Self {
        Row::new(&rrow.cells).style(rrow.style)
    }
}

impl<'a> From<&'a ResultCell> for Cell<'a> {
    fn from(rcell: &'a ResultCell) -> Self {
        let text = Into::<Text<'a>>::into(rcell.content.as_str()).alignment(rcell.alignment);
        Cell::new(text).style(rcell.style)
    }
}

impl From<ResultHeaderCell> for ResultCell {
    fn from(mut rhead: ResultHeaderCell) -> Self {
        let (padding, content) = if let Some(status) = rhead.status {
            ("  ", format!("{} {}", rhead.content, status))
        } else {
            ("", rhead.content.clone())
        };
        let content = match rhead.cell.alignment {
            Alignment::Left => content.clone(),
            Alignment::Center => format!("{}{}", padding, content),
            Alignment::Right => content.clone(),
        };
        rhead.cell.content = content;
        rhead.cell
    }
}

impl<C> From<C> for ResultRow
where
    C: IntoIterator,
    C::Item: Into<ResultCell>,
{
    fn from(cells: C) -> Self {
        Self {
            cells: cells.into_iter().map(Into::into).collect(),
            ..Default::default()
        }
    }
}

impl<S: Into<String>> From<S> for ResultCell {
    fn from(s: S) -> Self {
        Self {
            content: s.into(),
            ..Default::default()
        }
    }
}

impl<S: Into<ResultCell>> From<(S, Option<char>)> for ResultHeaderCell {
    fn from(s: (S, Option<char>)) -> Self {
        Self {
            cell: s.0.into(),
            status: s.1,
        }
    }
}
