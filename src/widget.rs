use std::{cmp::min, slice::Iter};

use crossterm::event::Event;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize as _},
    widgets::{Block, Borders, Clear, ScrollbarState, TableState},
    Frame,
};

use crate::app::App;

use self::theme::Theme;

pub mod category;
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

pub trait Widget {
    fn draw(&self, f: &mut Frame, app: &App, area: Rect);
    fn handle_event(&mut self, app: &mut App, e: &Event);
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

pub fn create_block<'a>(theme: &Theme, focused: bool) -> Block<'a> {
    Block::new()
        .border_style(match focused {
            true => Style::new().fg(theme.border_focused_color),
            false => Style::new().fg(theme.border_color),
        })
        .bg(theme.bg)
        .fg(theme.fg)
        .borders(Borders::ALL)
        .border_type(theme.border)
}

pub fn clear(f: &mut Frame, area: Rect, fill: Color) {
    f.render_widget(Clear, area);
    f.render_widget(Block::new().bg(fill), area);
}

pub struct StatefulTable<T> {
    pub state: TableState,
    pub scrollbar_state: ScrollbarState,
    pub items: Vec<T>,
}

impl<T> StatefulTable<T> {
    pub fn with_items(items: Vec<T>) -> StatefulTable<T> {
        StatefulTable {
            state: TableState::new().with_selected(Some(0)),
            scrollbar_state: ScrollbarState::new(items.len()),
            items,
        }
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
}
