use bitflags::bitflags;
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::{Offset, Rect};

bitflags! {
    #[derive(Clone, Copy)]
    pub struct DragEdge: u8 {
        const TOP = 0x01;
        const BOTTOM = 0x02;
        const LEFT = 0x04;
        const RIGHT = 0x08;

        const ALL = Self::TOP.bits() | Self::BOTTOM.bits() | Self::LEFT.bits() | Self::RIGHT.bits();
    }
}

#[derive(Default)]
pub struct DragState {
    offset: (i32, i32),
    start_click: Option<(u16, u16)>,
    last_area: Rect,
    edge: Option<DragEdge>,
}

impl DragState {
    pub fn edge(edge: DragEdge) -> Self {
        Self {
            edge: Some(edge),
            ..Default::default()
        }
    }

    pub fn reset(&mut self) {
        self.offset = (0, 0);
        self.start_click = None;
    }

    pub fn last_area(&self) -> Rect {
        self.last_area
    }

    pub fn on_mouse(&mut self, event: &MouseEvent) {
        if let MouseEventKind::Drag(MouseButton::Left) = event.kind {
            if let Some(start_click) = self.start_click {
                let to_pos = (event.column as i32, event.row as i32);
                let from_pos = (start_click.0 as i32, start_click.1 as i32);
                self.start_click = Some((event.column, event.row));
                self.offset = (
                    self.offset.0 + (from_pos.0 - to_pos.0),
                    self.offset.1 + (from_pos.1 - to_pos.1),
                );
            }
            return;
        }

        if let Some(edge) = self.edge {
            let top = edge.contains(DragEdge::TOP) && event.row == self.last_area.top();
            let bottom = edge.contains(DragEdge::BOTTOM)
                && event.row == self.last_area.bottom().saturating_sub(1);
            let left = edge.contains(DragEdge::LEFT) && event.column == self.last_area.left();
            let right = edge.contains(DragEdge::RIGHT)
                && event.column == self.last_area.right().saturating_sub(1);

            if !top && !bottom && !left && !right {
                return;
            }
        } else if !self.last_area.contains((event.column, event.row).into()) {
            return;
        }

        if let MouseEventKind::Down(MouseButton::Left) = event.kind {
            self.start_click = Some((event.column, event.row));
        } else {
            self.start_click = None;
        }
    }

    pub fn set_last_area(&mut self, last_area: Rect) {
        self.last_area = last_area;
    }

    pub fn drag(&self, area: Rect, screen: Rect) -> Rect {
        let mut offset = area.offset(Offset {
            x: -self.offset.0,
            y: -self.offset.1,
        });

        offset.x = offset
            .x
            .clamp(screen.x, screen.x + screen.width.saturating_sub(area.width));

        offset.y = offset.y.clamp(
            screen.y,
            screen.y + screen.height.saturating_sub(area.height),
        );

        offset
    }
}

pub trait DragExt {
    fn drag(&self, drag_state: &DragState, screen: Rect) -> Rect;
}

impl DragExt for Rect {
    fn drag(&self, drag_state: &DragState, screen: Rect) -> Rect {
        drag_state.drag(*self, screen)
    }
}
