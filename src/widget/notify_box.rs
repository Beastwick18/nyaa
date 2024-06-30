use ratatui::{
    layout::{Offset, Rect},
    style::Stylize as _,
    widgets::{Block, Borders, Paragraph, Widget as _},
    Frame,
};

use crate::{app::Context, style};

use super::Corner;

impl Corner {
    fn is_top(&self) -> bool {
        matches!(self, Self::TopLeft | Self::TopRight)
    }

    fn is_left(&self) -> bool {
        matches!(self, Self::TopLeft | Self::BottomLeft)
    }

    fn get_start_stop(
        self,
        area: Rect,
        width: u16,
        height: u16,
        start_offset: u16,
        stop_offset: u16,
    ) -> ((i32, i32), (i32, i32), (i32, i32)) {
        let stop_x = match self.is_left() {
            true => area.left() as i32 - width as i32,
            false => area.right() as i32 + 1,
        };
        let start_x = match self.is_left() {
            true => area.left() as i32 + 1,
            false => area.right() as i32 - width as i32 - 1,
        };
        let start_y = match self.is_top() {
            true => area.top() as i32 - height as i32 + start_offset as i32 + 2,
            false => area.bottom() as i32 - start_offset as i32 - 1,
        };
        let stop_y = match self.is_top() {
            true => area.top() as i32 + stop_offset as i32 + 2,
            false => area.bottom() as i32 - stop_offset as i32 - height as i32 - 1,
        };
        ((start_x, start_y), (start_x, stop_y), (stop_x, stop_y))
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

    pub fn translate(
        &mut self,
        func: fn(f64) -> f64,
        start_pos: (i32, i32),
        stop_pos: (i32, i32),
        rate: f64,
        deltatime: f64,
    ) -> (i32, i32) {
        if self.time >= 1.0 {
            self.done = true;
        }
        let pos = (
            ((func(self.time) * (stop_pos.0 - start_pos.0) as f64) + start_pos.0 as f64).round()
                as i32,
            ((func(self.time) * (stop_pos.1 - start_pos.1) as f64) + start_pos.1 as f64).round()
                as i32,
        );
        self.time = 1.0_f64.min(self.time + rate * deltatime);
        pos
    }

    pub fn ease_out(
        &mut self,
        start_pos: (i32, i32),
        stop_pos: (i32, i32),
        rate: f64,
        deltatime: f64,
    ) -> (i32, i32) {
        self.translate(Self::_ease_out, start_pos, stop_pos, rate, deltatime)
    }

    pub fn ease_in(
        &mut self,
        start_pos: (i32, i32),
        stop_pos: (i32, i32),
        rate: f64,
        deltatime: f64,
    ) -> (i32, i32) {
        self.translate(Self::_ease_in, start_pos, stop_pos, rate, deltatime)
    }

    fn _ease_out(x: f64) -> f64 {
        1.0 - (1.0 - x).powi(3)
    }

    fn _ease_in(x: f64) -> f64 {
        x.powi(3)
    }

    fn is_done(self) -> bool {
        self.done
    }

    fn reset(&mut self) {
        self.time = 0.0;
        self.done = false;
    }
}

pub struct NotifyBox {
    raw_content: String,
    pub time: f64,
    pub duration: f64,
    animation_speed: f64,
    max_width: u16,
    position: Corner,
    width: u16,
    height: u16,
    start_offset: u16,
    stop_offset: u16,
    enter_state: AnimateState,
    leave_state: AnimateState,
    pub pos: Option<(i32, i32)>,
    error: bool,
}

impl NotifyBox {
    pub fn new(
        content: String,
        duration: f64,
        position: Corner,
        animation_speed: f64,
        max_width: u16,
        error: bool,
    ) -> Self {
        let raw_content = content.clone();
        let lines = textwrap::wrap(&content, max_width as usize);
        let actual_width = lines.iter().fold(0, |acc, x| acc.max(x.len())) as u16 + 2;
        let height = lines.len() as u16 + 2;
        NotifyBox {
            width: actual_width,
            height,
            raw_content,
            position,
            animation_speed,
            max_width,
            start_offset: 0,
            stop_offset: 0,
            time: 0.0,
            duration,
            enter_state: AnimateState::new(),
            leave_state: AnimateState::new(),
            pos: None,
            error,
        }
    }

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn height(&self) -> u16 {
        self.height
    }

    pub fn offset(&self) -> u16 {
        self.stop_offset
    }

    pub fn is_done(&self) -> bool {
        self.leave_state.is_done()
    }

    pub fn is_leaving(&self) -> bool {
        self.time >= 1.0
    }

    pub fn is_error(&self) -> bool {
        self.error
    }

    pub fn add_offset<I: Into<i32> + Copy>(&mut self, offset: I) {
        self.enter_state.reset();

        self.start_offset = self.stop_offset + self.height;
        self.stop_offset = (self.stop_offset as i32 + Into::<i32>::into(offset)).max(0) as u16;
    }

    pub fn draw(&mut self, f: &mut Frame, ctx: &Context, area: Rect) {
        let max_width = match self.error {
            true => (area.width / 3).max(self.max_width),
            false => area.width.min(self.max_width),
        } as usize;
        let lines = textwrap::wrap(&self.raw_content, max_width);
        self.width = lines.iter().fold(0, |acc, x| acc.max(x.len())) as u16 + 2;
        self.height = lines.len() as u16 + 2;
        let content = lines.join("\n");

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
        let block = match self.error {
            false => Block::new()
                .border_style(style!(fg:ctx.theme.border_focused_color))
                .bg(ctx.theme.bg)
                .fg(ctx.theme.fg)
                .borders(border)
                .border_type(ctx.theme.border),
            true => {
                let mut block = Block::new()
                    .border_style(style!(fg:ctx.theme.error))
                    .bg(ctx.theme.bg)
                    .fg(ctx.theme.error)
                    .borders(border)
                    .border_type(ctx.theme.border);
                if border.contains(Borders::TOP) {
                    let title = "Error: Press ESC to dismiss...";
                    if let Some(sub) = title.get((scroll_x as usize)..) {
                        block = block.title(sub);
                    }
                }
                block
            }
        };

        super::clear(rect, f.buffer_mut(), ctx.theme.bg);
        Paragraph::new(content)
            .block(block)
            .scroll((scroll_y, scroll_x))
            .render(rect, f.buffer_mut());
    }

    fn next_pos(&mut self, deltatime: f64, area: Rect) -> (i32, i32) {
        let (start_pos, stop_pos, leave_pos) = self.position.get_start_stop(
            area,
            self.width,
            self.height,
            self.start_offset,
            self.stop_offset,
        );
        if self.time < 1.0 {
            self.enter_state
                .ease_out(start_pos, stop_pos, self.animation_speed, deltatime)
        } else {
            self.leave_state
                .ease_in(stop_pos, leave_pos, self.animation_speed / 2.0, deltatime)
        }
    }

    pub fn update(&mut self, deltatime: f64, area: Rect) -> bool {
        let last_pos = self.pos;
        self.pos = Some(self.next_pos(deltatime, area));

        // Dont automatically dismiss errors
        if self.enter_state.is_done() && !self.error {
            self.time = 1.0_f64.min(self.time + deltatime / self.duration);
        }
        last_pos != self.pos
    }
}
