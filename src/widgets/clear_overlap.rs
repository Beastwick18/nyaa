use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use unicode_width::UnicodeWidthStr as _;

/// Performs the same function as ratatui::widgets::Clear, except it removes wide
/// characters which would overlap the given area.
///
/// ```rs
/// ClearOverlap.render(area, buf);
/// ```
#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
pub struct ClearOverlap;

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
    }
}
