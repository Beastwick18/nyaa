use std::slice::Iter;

use crossterm::event::Event;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::TableState,
    Frame,
};

use crate::app::App;

use self::theme::Theme;

pub mod category;
pub mod filter;
pub mod results;
pub mod search;
pub mod sort;
pub mod theme;

pub trait Popup {
    fn draw(&self, f: &mut Frame, theme: &Theme);
    fn handle_event(&mut self, app: &mut App, e: &Event);
}

pub trait Widget {
    fn draw(&self, f: &mut Frame, app: &App, area: Rect);
    fn handle_event(&mut self, app: &mut App, e: &Event);
}

pub trait EnumIter<T> {
    fn iter() -> Iter<'static, T>;
}

pub fn centered_rect(x_len: u16, y_len: u16, r: Rect) -> Rect {
    let popup_layout = Layout::new(
        Direction::Vertical,
        &[
            Constraint::Length((r.height - y_len) / 2),
            Constraint::Length(y_len),
            Constraint::Length((r.height - y_len) / 2),
        ],
    )
    .split(r);

    Layout::new(
        Direction::Horizontal,
        &[
            Constraint::Length((r.width - x_len) / 2),
            Constraint::Length(x_len),
            Constraint::Length((r.width - x_len) / 2),
        ],
    )
    .split(popup_layout[1])[1]
}

pub struct StatefulTable<T> {
    pub state: TableState,
    pub items: Vec<T>,
}

impl<T> StatefulTable<T> {
    pub fn with_items(items: Vec<T>) -> StatefulTable<T> {
        StatefulTable {
            state: TableState::new().with_selected(Some(0)),
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
    }

    pub fn next(&mut self, amt: isize) {
        if self.items.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => i as isize + amt,
            None => 0,
        };
        self.state
            .select(Some(i.max(0).min(self.items.len() as isize - 1) as usize));
    }

    pub fn select(&mut self, idx: usize) {
        self.state.select(Some(idx));
    }
}
