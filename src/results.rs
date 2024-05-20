use ratatui::{
    layout::{Alignment, Constraint},
    style::{Style, Stylize},
    text::Span,
    widgets::{Cell, Row},
};

use crate::{source::Item, widget::sort::SortDir};

pub struct ResultHeader<S: PartialEq + Copy> {
    cols: Vec<ResultColumn<S>>,
}

pub enum ResultColumn<S: PartialEq + Copy> {
    Normal(String, Constraint),
    Sorted(String, u16, S),
}

impl<S: PartialEq + Copy> ResultHeader<S> {
    pub fn new<T>(cols: T) -> Self
    where
        T: IntoIterator,
        T::Item: Into<ResultColumn<S>>,
    {
        Self {
            cols: cols.into_iter().map(Into::into).collect(),
        }
    }

    pub fn get_row(&self, dir: SortDir, sort_by: S) -> ResultRow {
        ResultRow::new(
            self.cols
                .iter()
                .map(|c| c.get_render(dir, sort_by))
                .collect::<Vec<String>>(),
        )
    }

    pub fn get_binding(&self) -> Vec<Constraint> {
        self.cols
            .iter()
            .map(|c| match c {
                ResultColumn::Normal(_, c) => *c,
                ResultColumn::Sorted(_, l, _) => Constraint::Length(*l),
            })
            .collect()
    }
}

impl<S: PartialEq + Copy> ResultColumn<S> {
    pub fn get_render(&self, dir: SortDir, sort_by: S) -> String {
        match self {
            Self::Sorted(name, len, s) => {
                let mut name = format!("{:^width$}", name, width = *len as usize);
                if sort_by.eq(s) {
                    if let Some(idx) = name.rfind(|c: char| !c.is_whitespace()) {
                        if idx + 2 < name.len() {
                            name.replace_range(
                                name.char_indices()
                                    .nth(idx + 2)
                                    .map(|(pos, ch)| (pos..pos + ch.len_utf8()))
                                    .unwrap(),
                                match dir {
                                    SortDir::Asc => "▲",
                                    SortDir::Desc => "▼",
                                },
                            );
                        }
                    }
                }
                name
            }
            Self::Normal(name, _) => name.to_owned(),
        }
    }

    // pub fn is_sort(&self) -> bool {
    //     matches!(self, Self::Sorted(_, _))
    // }
    //
    // pub fn is_normal(&self) -> bool {
    //     matches!(self, Self::Normal(_, _))
    // }
}

#[derive(Default)]
pub struct ResultTable {
    pub headers: ResultRow,
    pub rows: Vec<ResultRow>,
    pub binding: Vec<Constraint>,
    pub items: Vec<Item>,
}

#[derive(Clone)]
pub struct ResultCell {
    pub content: String,
    pub style: Style,
}

impl<'a> From<ResultRow> for Row<'a> {
    fn from(val: ResultRow) -> Self {
        Row::new(val.cells)
    }
}

impl<'a> From<ResultCell> for Cell<'a> {
    fn from(val: ResultCell) -> Self {
        Cell::default().content(val.content).style(val.style)
    }
}

impl<'a> From<Span<'a>> for ResultCell {
    fn from(value: Span<'a>) -> Self {
        Self {
            content: value.content.to_string(),
            style: value.style,
        }
    }
}

impl From<String> for ResultCell {
    fn from(value: String) -> Self {
        Self {
            content: value,
            style: Style::default(),
        }
    }
}

impl From<&mut String> for ResultCell {
    fn from(value: &mut String) -> Self {
        Self {
            content: value.to_owned(),
            style: Style::default(),
        }
    }
}

#[derive(Default, Clone)]
pub struct ResultRow {
    pub cells: Vec<ResultCell>,
    pub style: Style,
}

impl<'a> Stylize<'a, ResultRow> for ResultRow {
    fn bg(self, color: ratatui::prelude::Color) -> Self {
        let mut newself = self;
        newself.style = newself.style.bg(color);
        newself
    }

    fn fg<S: Into<ratatui::prelude::Color>>(self, color: S) -> Self {
        let mut newself = self;
        newself.style = newself.style.fg(color.into());
        newself
    }

    fn reset(self) -> Self {
        let mut newself = self;
        newself.style = newself.style.reset();
        newself
    }

    fn add_modifier(self, modifier: ratatui::prelude::Modifier) -> Self {
        let mut newself = self;
        newself.style = newself.style.add_modifier(modifier);
        newself
    }

    fn remove_modifier(self, modifier: ratatui::prelude::Modifier) -> Self {
        let mut newself = self;
        newself.style = newself.style.remove_modifier(modifier);
        newself
    }
}

impl ResultRow {
    pub fn new<T>(cells: T) -> Self
    where
        T: IntoIterator,
        T::Item: Into<ResultCell>,
    {
        Self {
            cells: cells.into_iter().map(Into::into).collect(),
            style: Style::default(),
        }
    }

    pub fn aligned<A, B>(&mut self, align: A, binding: B) -> Self
    where
        A: IntoIterator,
        A::Item: Into<Alignment>,
        B: IntoIterator,
        B::Item: Into<Constraint>,
    {
        let align = align
            .into_iter()
            .map(Into::into)
            .collect::<Vec<Alignment>>();
        let binding = binding
            .into_iter()
            .map(Into::into)
            .collect::<Vec<Constraint>>();
        self.cells
            .iter_mut()
            .zip(align.iter())
            .zip(binding.iter())
            .for_each(|((c, a), b)| {
                c.content = match (a, b) {
                    (Alignment::Right, Constraint::Length(l)) => {
                        format!("{:>width$}", &c.content, width = *l as usize)
                    }
                    (Alignment::Center, Constraint::Length(l)) => {
                        format!("{:^width$}", &c.content, width = *l as usize)
                    }
                    _ => c.content.to_owned(),
                };
            });
        self.to_owned()
    }
}
