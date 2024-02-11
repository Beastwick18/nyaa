use std::slice::Iter;

use crossterm::event::Event;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

use crate::app::App;

pub mod category;
pub mod search;
pub mod sort;

pub trait Popup {
    fn draw(&self, f: &mut Frame);
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
