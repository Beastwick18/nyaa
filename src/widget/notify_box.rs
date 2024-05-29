use std::cmp::{max, min};

use ratatui::{
    layout::Rect,
    style::Stylize as _,
    widgets::{Block, Borders, Paragraph, Widget as _},
    Frame,
};

use crate::{app::Context, style};

const ANIM_SPEED: f64 = 8.0;
const MAX_WIDTH: u16 = 75;

pub struct NotifyBox {
    content: String,
    pub time: f64,
    pub duration: f64,
    width: u16,
    height: u16,
    offset: u16,
    enter_state: AnimateState,
    leave_state: AnimateState,
}

impl NotifyBox {
    pub fn new(content: String, duration: f64) -> Self {
        let width = (MAX_WIDTH - 2).min(content.len() as u16);
        let lines = textwrap::wrap(&content, width as usize);
        let actual_width = lines.iter().fold(0, |acc, x| acc.max(x.len())) as u16;
        let content = lines.join("\n");
        let height = lines.len() as u16 + 2;
        NotifyBox {
            width: actual_width + 2,
            height,
            content,
            offset: 0,
            time: 0.0,
            duration,
            enter_state: AnimateState::new(),
            leave_state: AnimateState::new(),
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
        let orig_width = self.width.min(area.width);
        let orig_height = self.height.min(area.height);

        let start_pos = (
            // (area.right() - width) as i32 - 2,
            area.right() as i32 + 2,
            // (area.top() as i32) + self.offset as i32, // Top aligned
            (area.bottom() as i32 - self.height as i32 - 1) - self.offset as i32,
        );
        let stop_pos = (
            (area.right() as i32 - orig_width as i32) - 2,
            // (area.top() as i32) + self.offset as i32,
            (area.bottom() as i32 - self.height as i32 - 1) - self.offset as i32,
        );
        let pos = match self.time >= 1.0 {
            false => self
                .enter_state
                .linear(start_pos, stop_pos, ANIM_SPEED, ctx.deltatime),
            true => self
                .leave_state
                .linear(stop_pos, start_pos, ANIM_SPEED, ctx.deltatime),
        };
        let mut border = Borders::ALL;
        let mut width = orig_width;
        let mut height = orig_height;
        if pos.0 < 0 {
            border &= !Borders::LEFT & Borders::ALL;
            width = max(0, width as i32 + min(pos.0, 0)) as u16;
        }
        if pos.0 + width as i32 > area.right() as i32 {
            border &= !Borders::RIGHT & Borders::ALL;
            width -= min((pos.0 + width as i32) - area.right() as i32, width as i32) as u16;
        }
        if pos.1 < 0 {
            border &= !Borders::TOP & Borders::ALL;
            height = max(0, height as i32 + min(pos.1, 0)) as u16;
        }
        if pos.1 + height as i32 > area.bottom() as i32 {
            border &= !Borders::BOTTOM & Borders::ALL;
            height -= min(
                (pos.1 + height as i32) - area.bottom() as i32,
                height as i32,
            ) as u16;
        }
        let rect = Rect::new(
            min(max(pos.0, 0) as u16, area.right() - width),
            min(max(pos.1, 0) as u16, area.bottom() - height),
            width,
            height,
        );
        let scroll_x = min(0, pos.0 + 1).unsigned_abs() as u16;
        let scroll_y = min(0, pos.1 + 1).unsigned_abs() as u16;
        let block = Block::new()
            .border_style(style!(fg:ctx.theme.border_focused_color))
            .bg(ctx.theme.bg)
            .fg(ctx.theme.fg)
            .borders(border)
            .border_type(ctx.theme.border);

        let clear_x = min(max(pos.0 - 1, 0) as u16, area.right() - width);
        let clear_width = (rect.width as i32 + 2)
            .min(area.right() as i32 - pos.0)
            .max(0) as u16;
        let clear = Rect::new(clear_x, rect.y, clear_width, rect.height);
        super::clear(clear, f.buffer_mut(), ctx.theme.bg);
        Paragraph::new(self.content.clone())
            .block(block)
            .scroll((scroll_y, scroll_x))
            .render(rect, f.buffer_mut());

        if self.enter_state.is_done() {
            self.time = 1.0_f64.min(self.time + ctx.deltatime / self.duration);
        }
    }
}
