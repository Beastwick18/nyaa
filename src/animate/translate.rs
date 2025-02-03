use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Clear, Widget},
};

use super::{Animation, AnimationState, FloatRect};

#[derive(Copy, Clone)]
pub struct Translate {
    state: AnimationState,
    start: Rect,
    stop: Rect,
}

impl Translate {
    pub fn new(state: AnimationState) -> Self {
        Self {
            state,
            start: Rect::default(),
            stop: Rect::default(),
        }
    }
}

impl From<AnimationState> for Translate {
    fn from(state: AnimationState) -> Self {
        Self::new(state)
    }
}

impl Translate {
    pub fn start(mut self, start: Rect) -> Self {
        self.start = start;
        self
    }

    pub fn stop(mut self, stop: Rect) -> Self {
        self.stop = stop;
        self
    }

    pub fn then<'a>(&'a mut self, other: &'a mut Translate) -> &'a mut Translate {
        if self.state.is_playing() {
            self
        } else {
            other
        }
    }

    fn translate(from: f64, to: f64, time: f64) -> f64 {
        time * (to - from) + from
    }

    fn translate_rect(&self, from: FloatRect, to: FloatRect) -> FloatRect {
        let time = self.state.get_smooth_time();

        let x = Self::translate(from.x, to.x, time);
        let y = Self::translate(from.y, to.y, time);
        let width = Self::translate(from.width, to.width, time);
        let height = Self::translate(from.height, to.height, time);

        FloatRect {
            x,
            y,
            width,
            height,
        }
    }

    pub fn area(&self) -> Rect {
        let time = self.state.get_smooth_time();

        // If we've already reached the end and are going forwards, stop
        if self.start == self.stop || time == 1.0 && self.state.speed >= 0.0 {
            self.stop
        }
        // If we've already reached the start and are going backwards, stop
        else if time == 0.0 && self.state.speed <= 0.0 {
            self.start
        }
        // Otherwise, translate
        else {
            self.translate_rect(self.start.into(), self.stop.into())
                .into()
        }
    }
}

impl Animation for Translate {
    fn state(&self) -> AnimationState {
        self.state
    }

    fn state_mut(&mut self) -> &mut AnimationState {
        &mut self.state
    }

    fn render_widget<W: Widget>(&self, widget: W, area: Rect, buf: &mut Buffer) {
        let rect = self.area().intersection(area);

        Clear.render(rect, buf);
        widget.render(rect, buf);
    }
}
