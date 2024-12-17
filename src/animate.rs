use ratatui::layout::Rect;

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
pub enum AnimationType {
    EaseInAndOut,
    // EaseIn,
    // EaseOut,
    #[default]
    Linear,
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

impl AnimationType {
    pub fn translate(self, from: f64, to: f64, time: f64) -> f64 {
        let easing_fn = match self {
            Self::EaseInAndOut => ease_in_out_quad,
            Self::Linear => linear,
        };
        easing_fn(time) * (to - from) + from
    }

    pub fn translate_rect(self, from: FloatRect, to: FloatRect, time: f64) -> FloatRect {
        let x = self.translate(from.x, to.x, time);
        let y = self.translate(from.y, to.y, time);
        let width = self.translate(from.width, to.width, time);
        let height = self.translate(from.height, to.height, time);

        FloatRect {
            x,
            y,
            width,
            height,
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct Animation {
    time: f64,
    speed: f64,
    running: bool,
    animation_type: AnimationType,
}

impl Animation {
    pub fn new(animation_type: AnimationType, speed: f64) -> Self {
        Self {
            animation_type,
            speed,
            running: false,
            time: 0.0,
        }
    }

    pub fn playing(mut self) -> Self {
        self.running = true;
        self
    }

    pub fn pausing(mut self) -> Self {
        self.running = false;
        self
    }

    pub fn reversed(mut self) -> Self {
        self.speed *= -1.0;
        self
    }

    pub fn play(&mut self) {
        self.running = true;
    }

    pub fn pause(&mut self) {
        self.running = false;
    }

    pub fn reverse(&mut self) {
        self.speed *= -1.0;
    }

    pub fn forwards(&mut self) {
        self.speed = self.speed.abs();
    }

    pub fn backwards(&mut self) {
        self.speed = -self.speed.abs();
    }

    pub fn reset(&mut self) {
        self.time = 0.0;
    }

    pub fn speed(&mut self, speed: f64) {
        self.speed = speed;
    }

    pub fn animate(self, start: Rect, stop: Rect) -> Rect {
        // If we've already reached the end and are going forwards, stop
        if start == stop || self.time == 1.0 && self.speed >= 0.0 {
            return stop;
        }
        // If we've already reached the start and are going backwards, stop
        if self.time == 0.0 && self.speed <= 0.0 {
            return start;
        }

        // Translate
        self.animation_type
            .translate_rect(start.into(), stop.into(), self.time)
            .into()
    }

    pub fn update(&mut self) {
        if !self.running {
            return;
        }
        self.time = (self.time + self.speed).max(0.0).min(1.0);
    }
}
