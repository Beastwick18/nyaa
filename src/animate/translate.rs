use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use super::{Animation, AnimationState, FloatRect, MaskedRenderer};

#[derive(Copy, Clone)]
pub struct Translate<'a> {
    state: &'a AnimationState,
    start: FloatRect,
    stop: FloatRect,
}

impl<'a> Translate<'a> {
    pub fn new(state: &'a AnimationState, start: FloatRect, stop: FloatRect) -> Self {
        Self { state, start, stop }
    }
}

impl<'a> Translate<'a> {
    /// Chain translations together
    pub fn then(&'a self, other: &'a Translate<'a>) -> &'a Translate<'a> {
        if !self.state.is_done() {
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

    pub fn area(&self) -> FloatRect {
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
            self.translate_rect(self.start, self.stop)
        }
    }
}

impl Animation for Translate<'_> {
    fn render_widget<W: Widget>(&self, widget: W, _area: Rect, buf: &mut Buffer) {
        MaskedRenderer::render(widget, self.area(), None, buf);
    }
}
