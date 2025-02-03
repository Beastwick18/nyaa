use ratatui::{buffer::Buffer, layout::Rect, style::Color, widgets::Widget};

use crate::animate::AnimationState;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Dim {
    dim: f64,
}

impl Dim {
    pub fn new(dim: f64) -> Self {
        Self { dim }
    }

    pub fn animated(mut self, state: AnimationState) -> Self {
        self.dim = 1.0 - (state.get_smooth_time() * self.dim);
        self
    }
}

impl Widget for Dim {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for x in area.left()..area.right() {
            for y in area.top()..area.bottom() {
                let cell = &mut buf[(x, y)];

                cell.bg = match cell.bg {
                    Color::Rgb(r, g, b) => Color::Rgb(
                        (r as f64 * self.dim) as u8,
                        (g as f64 * self.dim) as u8,
                        (b as f64 * self.dim) as u8,
                    ),
                    _ => cell.bg,
                };

                cell.fg = match cell.fg {
                    Color::Rgb(r, g, b) => Color::Rgb(
                        (r as f64 * self.dim) as u8,
                        (g as f64 * self.dim) as u8,
                        (b as f64 * self.dim) as u8,
                    ),
                    _ => cell.fg,
                };
            }
        }
    }
}
