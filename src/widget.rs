use std::{cmp::min, slice::Iter};

use crossterm::event::Event;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Stylize as _},
    widgets::{Block, Borders, Clear, ScrollbarState, TableState, Widget as _},
    Frame,
};

use crate::{app::Context, style};

use self::theme::Theme;

pub mod batch;
pub mod category;
pub mod clients;
pub mod error;
pub mod filter;
pub mod help;
pub mod input;
pub mod page;
pub mod results;
pub mod search;
pub mod sort;
pub mod sources;
pub mod theme;
pub mod user;

pub trait Widget {
    fn draw(&mut self, buf: &mut Frame, ctx: &Context, area: Rect);
    fn handle_event(&mut self, app: &mut Context, e: &Event);
    fn get_help() -> Option<Vec<(&'static str, &'static str)>>;
}

pub trait EnumIter<T> {
    fn iter() -> Iter<'static, T>;
}

pub fn centered_rect(mut x_len: u16, mut y_len: u16, r: Rect) -> Rect {
    x_len = min(x_len, r.width);
    y_len = min(y_len, r.height);
    let popup_layout = Layout::new(
        Direction::Vertical,
        [
            Constraint::Length((r.height - y_len) / 2),
            Constraint::Length(y_len),
            Constraint::Length((r.height - y_len) / 2),
        ],
    )
    .split(r);

    Layout::new(
        Direction::Horizontal,
        [
            Constraint::Length((r.width - x_len) / 2),
            Constraint::Length(x_len),
            Constraint::Length((r.width - x_len) / 2),
        ],
    )
    .split(popup_layout[1])[1]
}

pub fn border_block(theme: &Theme, focused: bool) -> Block {
    Block::new()
        .border_style(match focused {
            true => style!(fg:theme.border_focused_color),
            false => style!(fg:theme.border_color),
        })
        .bg(theme.bg)
        .fg(theme.fg)
        .borders(Borders::ALL)
        .border_type(theme.border)
}

pub fn clear(area: Rect, buf: &mut Buffer, fill: Color) {
    Clear.render(area, buf);
    Block::new().bg(fill).render(area, buf);
}

pub struct StatefulTable<T> {
    pub state: TableState,
    pub scrollbar_state: ScrollbarState,
    pub items: Vec<T>,
}

impl<T> StatefulTable<T> {
    pub fn new(items: Vec<T>) -> StatefulTable<T> {
        StatefulTable {
            state: TableState::default().with_selected(0),
            scrollbar_state: ScrollbarState::default(),
            items,
        }
    }

    pub fn empty() -> StatefulTable<T> {
        StatefulTable {
            state: TableState::default().with_selected(0),
            scrollbar_state: ScrollbarState::default(),
            items: vec![],
        }
    }

    pub fn with_items(&mut self, items: Vec<T>) -> &mut Self {
        self.state = TableState::new().with_selected(Some(0));
        self.scrollbar_state = ScrollbarState::new(items.len());
        self.items = items;
        self
    }

    pub fn next_wrap(&mut self, amt: isize) {
        if self.items.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => (i as isize + amt).rem_euclid(self.items.len() as isize),
            None => 0,
        };
        self.state.select(Some(i as usize));
        self.scrollbar_state = self.scrollbar_state.position(i as usize);
    }

    pub fn next(&mut self, amt: isize) {
        if self.items.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => i as isize + amt,
            None => 0,
        };
        let idx = i.max(0).min(self.items.len() as isize - 1) as usize;
        self.state.select(Some(idx));
        self.scrollbar_state = self.scrollbar_state.position(idx);
    }

    pub fn select(&mut self, idx: usize) {
        self.state.select(Some(idx));
        self.scrollbar_state = self.scrollbar_state.position(idx);
    }

    pub fn selected(&self) -> Option<&T> {
        self.state.selected().and_then(|i| self.items.get(i))
    }
}

pub struct VirtualStatefulTable {
    pub state: TableState,
    pub scrollbar_state: ScrollbarState,
}

impl VirtualStatefulTable {
    pub fn new() -> VirtualStatefulTable {
        VirtualStatefulTable {
            state: TableState::default().with_selected(0),
            scrollbar_state: ScrollbarState::default(),
        }
    }

    // pub fn next_wrap(&mut self, length: usize, amt: isize) {
    //     if length == 0 {
    //         return;
    //     }
    //     let i = match self.state.selected() {
    //         Some(i) => (i as isize + amt).rem_euclid(length as isize),
    //         None => 0,
    //     };
    //     self.state.select(Some(i as usize));
    //     self.scrollbar_state = self.scrollbar_state.position(i as usize);
    // }

    pub fn next(&mut self, length: usize, amt: isize) {
        if length == 0 {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => i as isize + amt,
            None => 0,
        };
        let idx = i.max(0).min(length as isize - 1) as usize;
        self.state.select(Some(idx));
        self.scrollbar_state = self.scrollbar_state.position(idx);
    }

    pub fn select(&mut self, idx: usize) {
        self.state.select(Some(idx));
        self.scrollbar_state = self.scrollbar_state.position(idx);
    }

    pub fn selected(&self) -> Option<usize> {
        self.state.selected()
    }
}
