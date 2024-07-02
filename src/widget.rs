use std::{cmp::min, slice::Iter};

use crossterm::event::Event;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Stylize as _},
    widgets::{
        Block, Borders, Clear, Scrollbar, ScrollbarOrientation, ScrollbarState, TableState,
        Widget as _,
    },
    Frame,
};
use serde::{Deserialize, Serialize};
use unicode_width::UnicodeWidthStr as _;

use crate::{app::Context, style, theme::Theme};

#[cfg(feature = "captcha")]
pub mod captcha;

pub mod batch;
pub mod category;
pub mod clients;
pub mod filter;
pub mod help;
pub mod input;
pub mod notifications;
pub mod notify_box;
pub mod page;
pub mod results;
pub mod search;
pub mod sort;
pub mod sources;
pub mod themes;
pub mod user;

pub trait Widget {
    fn draw(&mut self, buf: &mut Frame, ctx: &Context, area: Rect);
    fn handle_event(&mut self, app: &mut Context, e: &Event);
    fn get_help() -> Option<Vec<(&'static str, &'static str)>>;
}

pub trait EnumIter<T> {
    fn iter() -> Iter<'static, T>;
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum Corner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Corner {
    //pub fn try_title<'a, L: Into<Line<'a>>>(
    //    self,
    //    text: L,
    //    area: Rect,
    //    hide_if_too_small: bool,
    //) -> Option<(Line<'a>, Rect)> {
    //    let line: Line = text.into();
    //    let line_width = min(area.width, line.width() as u16);
    //    if hide_if_too_small && area.width < line.width() as u16 + 2 {
    //        // Too small
    //        return None;
    //    }
    //    let (left, y) = match self {
    //        Corner::TopLeft => (area.left() + 1, area.top()),
    //        Corner::TopRight => (area.right() - 1 - line_width, area.top()),
    //        Corner::BottomLeft => (area.left() + 1, area.bottom() - 1),
    //        Corner::BottomRight => (area.right() - 1 - line_width, area.bottom() - 1),
    //    };
    //    let right = Rect::new(left, y, line_width, 1);
    //    Some((line, right))
    //}
}

pub fn scroll_padding(
    selected: usize,
    height: usize,
    header_height: usize,
    num_items: usize,
    amt: usize,
    offset: &mut usize,
) {
    let first_row = *offset;
    let last_row = *offset + height;
    if selected + 1 == first_row + amt && selected >= amt {
        *offset -= 1;
    }
    if selected + amt + header_height == last_row && selected + amt != num_items {
        *offset += 1;
    }
}

pub fn dim_buffer(area: Rect, buf: &mut Buffer, amt: f32) {
    for r in area.top()..area.bottom() {
        for c in area.left()..area.right() {
            let cell = buf.get_mut(c, r);
            if let Color::Rgb(r, g, b) = cell.fg {
                let r = (r as f32 * amt) as u8;
                let g = (g as f32 * amt) as u8;
                let b = (b as f32 * amt) as u8;
                cell.fg = Color::Rgb(r, g, b);
            }
            if let Color::Rgb(r, g, b) = cell.bg {
                let r = (r as f32 * amt) as u8;
                let g = (g as f32 * amt) as u8;
                let b = (b as f32 * amt) as u8;
                cell.bg = Color::Rgb(r, g, b);
            }
        }
    }
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

pub fn scrollbar(ctx: &Context, orientation: ScrollbarOrientation) -> Scrollbar<'_> {
    let set = ctx.theme.border.to_border_set();
    let track = match orientation {
        ScrollbarOrientation::VerticalRight => set.vertical_right,
        ScrollbarOrientation::VerticalLeft => set.vertical_left,
        ScrollbarOrientation::HorizontalBottom => set.horizontal_bottom,
        ScrollbarOrientation::HorizontalTop => set.horizontal_top,
    };
    Scrollbar::default()
        .orientation(orientation)
        .track_symbol(Some(track))
        .begin_symbol(None)
        .end_symbol(None)
}

pub fn clear(area: Rect, buf: &mut Buffer, fill: Color) {
    // Deal with wide chars which might extend too far
    if area.left() > 0 && buf.area.contains((area.left() - 1, area.top()).into()) {
        for i in area.top()..area.bottom() {
            let c = buf.get_mut(area.left() - 1, i);
            if c.symbol().width() > 1 {
                c.set_char(' ');
            }
        }
    }
    Clear.render(area, buf);
    Block::new().bg(fill).render(area, buf);
}

pub struct StatefulTable<T> {
    pub state: TableState,
    pub scrollbar_state: ScrollbarState,
    pub items: Vec<T>,
}

impl<T: std::clone::Clone> StatefulTable<T> {
    pub fn new(items: &[T]) -> StatefulTable<T> {
        StatefulTable {
            state: TableState::default().with_selected(0),
            scrollbar_state: ScrollbarState::default(),
            items: items.to_vec(),
        }
    }

    pub fn empty() -> StatefulTable<T> {
        StatefulTable {
            state: TableState::default().with_selected(0),
            scrollbar_state: ScrollbarState::default(),
            items: vec![],
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

    // pub fn next(&mut self, amt: isize) {
    //     if self.items.is_empty() {
    //         return;
    //     }
    //     let i = match self.state.selected() {
    //         Some(i) => i as isize + amt,
    //         None => 0,
    //     };
    //     let idx = i.max(0).min(self.items.len() as isize - 1) as usize;
    //     self.state.select(Some(idx));
    //     self.scrollbar_state = self.scrollbar_state.position(idx);
    // }

    pub fn select(&mut self, idx: usize) {
        self.state.select(Some(idx));
        self.scrollbar_state = self.scrollbar_state.position(idx);
    }

    pub fn selected(&self) -> Option<&T> {
        self.state.selected().and_then(|i| self.items.get(i))
    }
}

#[derive(Default)]
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

    pub fn next_wrap(&mut self, length: usize, amt: isize) -> usize {
        if length == 0 {
            return 0;
        }
        let i = match self.state.selected() {
            Some(i) => (i as isize + amt).rem_euclid(length as isize),
            None => 0,
        } as usize;
        self.state.select(Some(i));
        self.scrollbar_state = self.scrollbar_state.position(i);
        i
    }

    pub fn next(&mut self, length: usize, amt: isize) -> usize {
        if length == 0 {
            return 0;
        }
        let idx = match self.state.selected() {
            Some(i) => i.saturating_add_signed(amt).min(length.saturating_sub(1)),
            None => 0,
        };
        self.state.select(Some(idx));
        self.scrollbar_state = self.scrollbar_state.position(idx);
        idx
    }

    pub fn select(&mut self, idx: usize) {
        self.state.select(Some(idx));
        self.scrollbar_state = self.scrollbar_state.position(idx);
    }

    pub fn selected(&self) -> Option<usize> {
        self.state.selected()
    }
}
