use ratatui::{
    buffer::Buffer,
    layout::{Position, Rect},
    widgets::Widget,
};
use serde::{Deserialize, Serialize};

use super::{Animation, AnimationState};

#[derive(Clone)]
pub struct GrowthAnimation {
    growth: Growth,
    state: AnimationState,
}

impl GrowthAnimation {
    pub fn new(state: AnimationState, growth: Growth) -> Self {
        Self { growth, state }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum Growth {
    // Random,
    Circle,
    Center,
    Top,
    Bottom,
    Left,
    Right,
}

impl Growth {
    fn should_skip(
        self,
        area: Rect,
        pos: Position,
        state: AnimationState,
        // seed: &mut SmallRng,
    ) -> bool {
        match self {
            // Growth::Random => {
            //     if seed.random_bool(1.0 - state.get_smooth_time()) {
            //         return true;
            //     }
            // }
            Growth::Circle => {
                let cx = area.x as f64 + area.width as f64 / 2.0;
                let cy = area.y as f64 + area.height as f64 / 2.0;
                if (((pos.x as f64 - cx) / 2.0).powi(2) + (pos.y as f64 - cy).powi(2)).sqrt()
                    > ((area.width as f64 / 4.0).powi(2) + (area.height as f64 / 2.0).powi(2))
                        .sqrt()
                        * state.get_smooth_time()
                {
                    return true;
                }
            }
            Growth::Center => {
                let hw = area.width as f64 / 2.0;
                if pos.x > (area.x as f64 + hw * (1.0 + state.get_smooth_time())) as u16 {
                    return true;
                }
                if pos.x < (area.x as f64 + hw * (1.0 - state.get_smooth_time())) as u16 {
                    return true;
                }
            }
            Growth::Top => {
                if pos.y > area.y + (area.height as f64 * state.get_smooth_time()) as u16 {
                    return true;
                }
            }
            Growth::Bottom => {
                if pos.y < area.y + (area.height as f64 * (1.0 - state.get_smooth_time())) as u16 {
                    return true;
                }
            }
            Growth::Left => {
                if pos.x > area.x + (area.width as f64 * state.get_smooth_time()) as u16 {
                    return true;
                }
            }
            Growth::Right => {
                if pos.x < area.x + (area.width as f64 * (1.0 - state.get_smooth_time())) as u16 {
                    return true;
                }
            }
        }
        false
    }
}

impl GrowthAnimation {
    pub fn then(self, other: GrowthAnimation) -> GrowthAnimation {
        if self.state.is_playing() {
            self
        } else {
            other
        }
    }
}

impl Animation for GrowthAnimation {
    fn render_widget<W: Widget>(&self, widget: W, rect: Rect, buf: &mut Buffer) {
        if self.state.time <= 0.0 {
            return;
        }
    }
}
