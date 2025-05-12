use std::time::Duration;

use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

pub mod growth;
pub mod transition;
pub mod translate;

pub trait Animation {
    fn render_widget<W: Widget>(&self, widget: W, rect: Rect, buf: &mut Buffer);
}

#[derive(Clone, Copy)]
pub struct MaskedRenderer;

impl MaskedRenderer {
    pub fn render(widget: impl Widget, area: FloatRect, mask: Option<&[bool]>, buf: &mut Buffer) {
        let (widget_x, screen_x) = if area.x < 0.0 {
            (
                0,
                (area.x.floor() as i32 + buf.area.x as i32).unsigned_abs() as u16,
            )
        } else {
            (area.x as u16, 0)
        };

        let (widget_y, screen_y) = if area.y < 0.0 {
            (
                0,
                (area.y.floor() as i32 + buf.area.y as i32).unsigned_abs() as u16,
            )
        } else {
            (area.y as u16, 0)
        };

        let widget_area = Rect::new(widget_x, widget_y, area.width as u16, area.height as u16);
        let screen_area = Rect::new(screen_x, screen_y, buf.area.width, buf.area.height);

        let widget_screen_intersection = widget_area.intersection(screen_area);

        // if not visible
        if widget_screen_intersection.is_empty() {
            return;
        }

        // w.xy < 0 ? w.xy = (0,0) & screen.xy = w.xy.abs()
        // +---+--------------+
        // | w |              |
        // +---+--------------+
        // |   |    offset    |
        // |   |    screen    |
        // |   |              |
        // +---+--------------+
        //
        // w.xy >= 0 ? w.xy = w.xy & screen.xy = screen.xy
        // +---+----------+
        // | w |          |
        // |---+screen    |
        // |              |
        // +--------------+
        //

        // Render entire widget to empty buffer
        let mut widget_buffer = Buffer::empty(widget_area);
        widget.render(widget_area, &mut widget_buffer);

        // skip some cells based on intersection
        let size = widget_buffer.area.area() as usize;
        for i in 0..size {
            let (x, y) = widget_buffer.pos_of(i);
            if !widget_screen_intersection.contains((x, y).into()) {
                continue;
            }

            let (x, y) = (x - screen_x, y - screen_y);
            let k = ((y.saturating_sub(buf.area.y)) * buf.area.width + x.saturating_sub(buf.area.x))
                as usize;
            buf.content[k] = widget_buffer.content[i].clone();
        }

        buf.area.x = 0;
        buf.area.y = 0;
    }
}

#[derive(Clone, Copy, Default, PartialEq)]
pub struct FloatRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl FloatRect {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

impl From<FloatRect> for Rect {
    fn from(rect: FloatRect) -> Self {
        Self {
            x: rect.x as u16,
            y: rect.y as u16,
            width: rect.width as u16,
            height: rect.height as u16,
        }
    }
}

impl From<&FloatRect> for Rect {
    fn from(rect: &FloatRect) -> Self {
        Self {
            x: rect.x as u16,
            y: rect.y as u16,
            width: rect.width as u16,
            height: rect.height as u16,
        }
    }
}

impl From<Rect> for FloatRect {
    fn from(rect: Rect) -> Self {
        Self {
            x: rect.x as f64,
            y: rect.y as f64,
            width: rect.width as f64,
            height: rect.height as f64,
        }
    }
}

#[derive(Clone, Copy, Default)]
pub enum Smoothing {
    EaseInAndOut,
    EaseOut,
    EaseIn,
    #[default]
    Linear,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum Direction {
    #[default]
    Forwards,
    Backwards,
}

impl From<Direction> for f64 {
    fn from(direction: Direction) -> Self {
        match direction {
            Direction::Forwards => 1.0,
            Direction::Backwards => -1.0,
        }
    }
}

fn ease_in_out_quad(x: f64) -> f64 {
    if x < 0.5 {
        2.0 * x * x
    } else {
        1.0 - (-2.0 * x + 2.0).powi(2) / 2.0
    }
}

fn linear(x: f64) -> f64 {
    x
}

fn ease_out(x: f64) -> f64 {
    1.0 - (1.0 - x).powi(3)
}

fn ease_in(x: f64) -> f64 {
    (x).powi(3)
}

impl Smoothing {
    pub fn apply(self, time: f64) -> f64 {
        let easing_fn = match self {
            Self::EaseInAndOut => ease_in_out_quad,
            Self::EaseOut => ease_out,
            Self::EaseIn => ease_in,
            Self::Linear => linear,
        };
        easing_fn(time)
    }
}

#[derive(Clone, Copy, Default)]
pub struct AnimationState {
    time: f64,
    speed: f64,
    direction: Direction,
    playing: bool,
    smoothing: Smoothing,
}

impl AnimationState {
    pub fn new(speed: f64) -> Self {
        Self {
            time: 0.0,
            speed: speed.abs(),
            direction: Direction::default(),
            playing: false,
            smoothing: Smoothing::default(),
        }
    }

    pub fn from_secs(secs: impl Into<f64>) -> Self {
        Self::new(1.0 / secs.into())
    }

    pub fn then<'a>(&'a mut self, other: &'a mut Self) -> &'a mut Self {
        if self.is_done() {
            other
        } else {
            self
        }
    }

    pub fn playing(mut self, playing: bool) -> Self {
        self.playing = playing;
        self
    }
    pub fn forwards(mut self) -> Self {
        self.direction = Direction::Forwards;
        self
    }
    pub fn backwards(mut self) -> Self {
        self.direction = Direction::Backwards;
        self
    }
    pub fn ending(mut self) -> Self {
        self.time = 1.0;
        self
    }
    pub fn smoothing(mut self, smoothing: Smoothing) -> Self {
        self.smoothing = smoothing;
        self
    }

    pub fn play(&mut self, playing: bool) {
        self.playing = playing;
    }
    pub fn set_direction(&mut self, direction: Direction) {
        self.direction = direction;
    }
    pub fn reverse(&mut self) {
        self.direction = match self.direction {
            Direction::Forwards => Direction::Backwards,
            Direction::Backwards => Direction::Forwards,
        };
    }
    pub fn goto_start(&mut self) {
        self.time = 0.0;
    }
    pub fn goto_end(&mut self) {
        self.time = 1.0;
    }
    pub fn set_speed(&mut self, speed: f64) {
        self.speed = speed.abs();
    }

    pub fn is_playing(&self) -> bool {
        self.playing
    }
    pub fn is_done(&self) -> bool {
        (self.time == 1.0 && self.direction == Direction::Forwards)
            || (self.time == 0.0 && self.direction == Direction::Backwards)
    }
    pub fn is_starting(&self) -> bool {
        self.time == 0.0
    }
    pub fn is_ending(&self) -> bool {
        self.time == 1.0
    }
    pub fn get_raw_time(&self) -> f64 {
        self.time
    }
    pub fn get_smooth_time(&self) -> f64 {
        self.smoothing.apply(self.time)
    }
    pub fn get_speed(&self) -> f64 {
        self.speed
    }
    pub fn get_direction(&self) -> Direction {
        self.direction
    }
    pub fn get_smoothing(&self) -> Smoothing {
        self.smoothing
    }

    pub fn update(&mut self, delta_time: f64) {
        if !self.playing {
            return;
        }
        self.time += delta_time * self.speed * Into::<f64>::into(self.direction);

        self.time = self.time.clamp(0.0, 1.0);
    }
}
