use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Stylize as _},
    widgets::{Block, Clear, Widget},
};
use unicode_width::UnicodeWidthStr as _;

/// Performs the same function as ratatui::widgets::Clear, except it removes wide
/// characters which would overlap the given area, and fills in the cleared space with a provided
/// fill color.
///
/// ```rs
/// ClearOverlap::default().render(area, buf);      // Defaults to Color::Reset
/// ClearOverlap::new(fillColor).render(area, buf); // Or pick a specific fill color
/// ```
#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
pub struct ClearOverlap {
    fill: Color,
}

impl ClearOverlap {
    pub fn new(fill: Color) -> Self {
        Self { fill }
    }
}

impl Widget for ClearOverlap {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.left() > 0 && buf.area.left() <= area.left() {
            for y in area.top()..area.bottom() {
                let cell = &mut buf[(area.left() - 1, y)];

                // Remove wide characters that may overlap area to be cleared
                if cell.symbol().width() > 1 {
                    cell.set_char(' ');
                }
            }
        }

        Clear.render(area, buf);

        Block::new().bg(self.fill).render(area, buf);
    }
}
