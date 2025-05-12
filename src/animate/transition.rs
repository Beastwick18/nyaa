use ratatui::style::Color;

use crate::color;

use super::AnimationState;

#[derive(Copy, Clone)]
pub struct Transition<'a> {
    state: &'a AnimationState,
    start: Color,
    stop: Color,
}

impl<'a> Transition<'a> {
    pub fn new(state: &'a AnimationState, start: Color, stop: Color) -> Self {
        Self { state, start, stop }
    }
}

impl<'a> Transition<'a> {
    fn transition(from: f64, to: f64, time: f64) -> f64 {
        time * (to - from) + from
    }

    fn transition_color(&self, from: impl Into<Color>, to: impl Into<Color>) -> Color {
        let time = self.state.get_smooth_time();

        let (from_r, from_g, from_b) = match color::to_rgb(from.into()) {
            Color::Rgb(r, g, b) => (r, g, b),
            _ => return Color::Reset,
        };

        let (to_r, to_g, to_b) = match color::to_rgb(to.into()) {
            Color::Rgb(r, g, b) => (r, g, b),
            _ => return Color::Reset,
        };

        let r = Self::transition(from_r as f64, to_r as f64, time);
        let g = Self::transition(from_g as f64, to_g as f64, time);
        let b = Self::transition(from_b as f64, to_b as f64, time);

        Color::Rgb(r as u8, g as u8, b as u8)
    }

    pub fn color(&self) -> Color {
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
            self.transition_color(self.start, self.stop)
        }
    }
}
