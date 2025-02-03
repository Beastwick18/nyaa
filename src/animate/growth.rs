use rand::{random, rngs::SmallRng, Rng as _, SeedableRng as _};
use ratatui::{
    buffer::Buffer,
    layout::{Position, Rect},
    widgets::Widget,
};
use serde::{Deserialize, Serialize};

use super::{Animation, AnimationState};

#[derive(Clone)]
pub struct GrowthAnimation {
    seed: SmallRng,
    growth: Growth,
    state: AnimationState,
}

impl GrowthAnimation {
    pub fn new(state: AnimationState, growth: Growth) -> Self {
        Self {
            seed: SmallRng::seed_from_u64(random()),
            growth,
            state,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum Growth {
    Random,
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
        seed: &mut SmallRng,
    ) -> bool {
        match self {
            Growth::Random => {
                if seed.gen_bool(1.0 - state.get_smooth_time()) {
                    return true;
                }
            }
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
    fn state(&self) -> AnimationState {
        self.state
    }

    fn state_mut(&mut self) -> &mut AnimationState {
        &mut self.state
    }

    fn render_widget<W: Widget>(&self, widget: W, rect: Rect, buf: &mut Buffer) {
        if self.state.time <= 0.0 {
            return;
        }

        let area = rect.intersection(buf.area);
        let mut other: Buffer = Buffer::empty(area);
        widget.render(area, &mut other);

        // Randomly merge more and more cells based on the animation time
        if self.state.time < 1.0 {
            let mut seed = self.seed.clone();
            let size = other.area.area() as usize;
            for i in 0..size {
                let (x, y) = other.pos_of(i);

                // Skip some cells
                if self
                    .growth
                    .should_skip(other.area, (x, y).into(), self.state, &mut seed)
                {
                    continue;
                }

                // New index in content
                let k = ((y.saturating_sub(buf.area.y)) * buf.area.width
                    + x.saturating_sub(buf.area.x)) as usize;
                buf.content[k] = other.content[i].clone();
            }
        } else {
            buf.merge(&other);
        }
    }
}
