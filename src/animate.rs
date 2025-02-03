use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

pub mod growth;
pub mod translate;

pub trait Animation {
    fn state(&self) -> AnimationState;

    fn state_mut(&mut self) -> &mut AnimationState;

    fn render_widget<W: Widget>(&self, widget: W, rect: Rect, buf: &mut Buffer);
}

#[derive(Clone, Copy, Default)]
pub struct FloatRect {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
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
    #[default]
    Linear,
}

#[derive(Clone, Copy, Default)]
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

impl Smoothing {
    pub fn apply(self, time: f64) -> f64 {
        let easing_fn = match self {
            Self::EaseInAndOut => ease_in_out_quad,
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
    pub fn finishing(mut self) -> Self {
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
    pub fn restart(&mut self) {
        self.time = 0.0;
    }
    pub fn finish(&mut self) {
        self.time = 1.0;
    }
    pub fn set_speed(&mut self, speed: f64) {
        self.speed = speed.abs();
    }

    pub fn is_playing(&self) -> bool {
        self.playing
    }
    pub fn is_done(&self) -> bool {
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

    // pub fn animate(self, start: Rect, stop: Rect) -> Rect {
    //     // If we've already reached the end and are going forwards, stop
    //     if start == stop || self.time == 1.0 && self.speed >= 0.0 {
    //         return stop;
    //     }
    //     // If we've already reached the start and are going backwards, stop
    //     if self.time == 0.0 && self.speed <= 0.0 {
    //         return start;
    //     }
    //
    //     // Translate
    //     self.smoothing
    //         .translate_rect(start.into(), stop.into(), self.time)
    //         .into()
    // }
    //
    pub fn update(&mut self) {
        if !self.playing {
            return;
        }
        self.time += self.speed * Into::<f64>::into(self.direction);

        self.time = self.time.max(0.0).min(1.0);
    }
}
