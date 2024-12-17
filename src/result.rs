use derive_more::{Deref, DerefMut};
use ratatui::{
    layout::{Alignment, Constraint},
    style::Style,
    text::Text,
    widgets::{Cell, Row, Table},
};

#[derive(Clone, Default, Deref, DerefMut)]
pub struct ResultHeader {
    #[deref]
    #[deref_mut]
    cell: ResultCell,
    status: char,
}

#[derive(Clone, Default)]
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

#[derive(Clone, Default)]
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

#[derive(Clone, Default)]
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
        R::Item: Into<ResultHeader>,
    {
        self.header = header
            .into_iter()
            .map(Into::<ResultHeader>::into)
            .map(Into::<ResultCell>::into)
            .collect::<Vec<ResultCell>>()
            .into();
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

impl<'a> From<ResultTable> for Table<'a> {
    fn from(res: ResultTable) -> Self {
        Table::new(res.rows, res.binding).header(res.header.into())
    }
}

impl<'a> From<ResultRow> for Row<'a> {
    fn from(rrow: ResultRow) -> Self {
        Row::new(rrow.cells).style(rrow.style)
    }
}

impl<'a> From<ResultCell> for Cell<'a> {
    fn from(rcell: ResultCell) -> Self {
        let text = Into::<Text<'a>>::into(rcell.content).alignment(rcell.alignment);
        Cell::new(text).style(rcell.style)
    }
}

impl From<ResultHeader> for ResultCell {
    fn from(mut rhead: ResultHeader) -> Self {
        let content = match rhead.cell.alignment {
            Alignment::Left => format!("{} {}", rhead.cell.content, rhead.status),
            Alignment::Center => format!("  {} {}", rhead.cell.content, rhead.status),
            Alignment::Right => todo!("{} {}", rhead.status, rhead.cell.content),
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

impl<S: Into<String>> From<(S, char)> for ResultHeader {
    fn from(s: (S, char)) -> Self {
        Self {
            cell: ResultCell::new(s.0),
            status: s.1,
        }
    }
}
