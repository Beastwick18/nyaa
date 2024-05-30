use ratatui::{
    layout::{Offset, Rect},
    style::Stylize as _,
    widgets::{Block, Borders, Paragraph, Widget as _},
    Frame,
};
use serde::{Deserialize, Serialize};

use crate::{app::Context, style};

const ANIM_SPEED: f64 = 8.0;
const MAX_WIDTH: u16 = 75;

#[derive(Clone, Copy, Deserialize, Serialize)]
pub enum NotifyPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl NotifyPosition {
    pub fn is_top(self) -> bool {
        matches!(self, Self::TopLeft | Self::TopRight)
    }

    pub fn is_left(self) -> bool {
        matches!(self, Self::TopLeft | Self::BottomLeft)
    }

    pub fn get_start_stop(
        self,
        area: Rect,
        width: u16,
        height: u16,
        offset: u16,
    ) -> ((i32, i32), (i32, i32)) {
        let start_x = if self.is_left() {
            area.left() as i32 - width as i32
        } else {
            area.right() as i32 + 1
        };
        let stop_x = if self.is_left() {
            area.left() as i32 + 2
        } else {
            area.right() as i32 - width as i32 - 2
        };
        // let start_y = if self.is_top() {
        //     (area.top() as i32 + 4) + offset as i32 - (height / 2) as i32
        // } else {
        //     (area.bottom() as i32 - 1) - offset as i32
        // };
        let stop_y = if self.is_top() {
            (area.top() as i32 + 4) + offset as i32
        } else {
            (area.bottom() as i32 - height as i32 - 1) - offset as i32
        };
        ((start_x, stop_y), (stop_x, stop_y))
    }
}

pub struct NotifyBox {
    content: String,
    pub time: f64,
    pub duration: f64,
    position: NotifyPosition,
    width: u16,
    height: u16,
    offset: u16,
    enter_state: AnimateState,
    leave_state: AnimateState,
    pos: Option<(i32, i32)>,
}

impl NotifyBox {
    pub fn new(content: String, duration: f64, position: NotifyPosition) -> Self {
        let width = (MAX_WIDTH - 2).min(content.len() as u16);
        let lines = textwrap::wrap(&content, width as usize);
        let actual_width = lines.iter().fold(0, |acc, x| acc.max(x.len())) as u16;
        let content = lines.join("\n");
        let height = lines.len() as u16 + 2;
        NotifyBox {
            width: actual_width + 2,
            height,
            content,
            position,
            offset: 0,
            time: 0.0,
            duration,
            enter_state: AnimateState::new(),
            leave_state: AnimateState::new(),
            pos: None,
        }
    }

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn height(&self) -> u16 {
        self.height
    }

    pub fn offset(&self) -> u16 {
        self.offset
    }

    pub fn with_offset(&mut self, offset: u16) -> &mut Self {
        self.offset = offset;
        self
    }
}

#[derive(Copy, Clone)]
pub struct AnimateState {
    time: f64,
    done: bool,
}

impl AnimateState {
    fn new() -> Self {
        Self {
            time: 0.0,
            done: false,
        }
    }

    fn linear(
        &mut self,
        start_pos: (i32, i32),
        stop_pos: (i32, i32),
        rate: f64,
        deltatime: f64,
    ) -> (i32, i32) {
        if self.time >= 1.0 {
            self.done = true;
        }
        let pos = (
            ((self.time * (stop_pos.0 - start_pos.0) as f64) + start_pos.0 as f64) as i32,
            ((self.time * (stop_pos.1 - start_pos.1) as f64) + start_pos.1 as f64) as i32,
        );
        self.time = 1.0_f64.min(self.time + rate * deltatime);
        pos
    }

    fn is_done(self) -> bool {
        self.done
    }
}

impl NotifyBox {
    pub fn is_done(&self) -> bool {
        self.leave_state.is_done()
    }

    pub fn is_leaving(&self) -> bool {
        self.time >= 1.0
    }

    pub fn draw(&mut self, f: &mut Frame, ctx: &Context, area: Rect) {
        let pos = self.pos.unwrap_or(self.next_pos(ctx.deltatime, area));
        let offset = Offset {
            x: self.width as i32,
            y: self.height as i32,
        };
        let offset_back = Offset {
            x: -(self.width as i32),
            y: -(self.height as i32),
        };
        let rect = Rect::new(
            (pos.0 + self.width as i32).max(0) as u16,
            (pos.1 + self.height as i32).max(0) as u16,
            self.width,
            self.height,
        )
        .intersection(area.offset(offset))
        .offset(offset_back);
        let mut border = Borders::NONE;
        if pos.0 >= 0 {
            border |= Borders::LEFT
        }
        if pos.0 + self.width as i32 <= area.right() as i32 {
            border |= Borders::RIGHT
        }
        if pos.1 >= 0 {
            border |= Borders::TOP
        }
        if pos.1 + self.height as i32 <= area.bottom() as i32 {
            border |= Borders::BOTTOM
        }
        let scroll_x = (pos.0 + 1).min(0).unsigned_abs() as u16;
        let scroll_y = (pos.1 + 1).min(0).unsigned_abs() as u16;
        let block = Block::new()
            .border_style(style!(fg:ctx.theme.border_focused_color))
            .bg(ctx.theme.bg)
            .fg(ctx.theme.fg)
            .borders(border)
            .border_type(ctx.theme.border);

        let clear = Rect::new(
            (pos.0 - 1) as u16,
            pos.1 as u16,
            self.width + 2,
            self.height,
        )
        .intersection(area);
        super::clear(clear, f.buffer_mut(), ctx.theme.bg);
        Paragraph::new(self.content.clone())
            .block(block)
            .scroll((scroll_y, scroll_x))
            .render(rect, f.buffer_mut());
    }

    fn next_pos(&mut self, deltatime: f64, area: Rect) -> (i32, i32) {
        let (start_pos, stop_pos) =
            self.position
                .get_start_stop(area, self.width, self.height, self.offset);
        match self.time >= 1.0 {
            false => self
                .enter_state
                .linear(start_pos, stop_pos, ANIM_SPEED, deltatime),
            true => self
                .leave_state
                .linear(stop_pos, start_pos, ANIM_SPEED, deltatime),
        }
    }

    pub fn update(&mut self, deltatime: f64, area: Rect) -> bool {
        let last_pos = self.pos;
        self.pos = Some(self.next_pos(deltatime, area));
        if self.enter_state.is_done() {
            self.time = 1.0_f64.min(self.time + deltatime / self.duration);
        }
        last_pos != self.pos
    }
}
