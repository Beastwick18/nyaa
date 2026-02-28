use color_eyre::Result;
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::{
    Frame,
    layout::{Margin, Rect},
    style::{Color, Style, Stylize as _},
    symbols::{line, scrollbar},
    text::Line,
    widgets::{Block, Borders, List, ListState, Widget as _},
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    action::{AppAction, UserAction},
    animate::{
        Animation, AnimationState, Direction, Smoothing,
        translate::{self, Translate},
    },
    app::{Context, Mode},
    color::ColorRgbExt,
    components,
    mouse::drag::{DragEdge, DragExt, DragState},
    sources::{SourceTask, query::sort::SortDirection},
    widgets::clear_overlap::ClearOverlap,
};

use super::Component;

pub struct Sorts {
    translate_state: AnimationState,
    drag: DragState,
    list: ListState,
    sort_dir: SortDirection,
}

impl Sorts {
    pub fn boxed() -> Box<dyn Component> {
        Box::new(Self {
            translate_state: AnimationState::new(6.0)
                .playing(true)
                .backwards()
                .smoothing(Smoothing::EaseInAndOut),
            drag: DragState::edge(DragEdge::ALL),
            list: ListState::default().with_selected(Some(0)),
            sort_dir: SortDirection::default(),
        })
    }
}

impl Component for Sorts {
    fn update(
        &mut self,
        ctx: &Context,
        action: &AppAction,
        action_tx: UnboundedSender<AppAction>,
    ) -> Result<()> {
        if action == &AppAction::Render {
            self.translate_state.set_direction(match ctx.mode {
                Mode::Sorts => Direction::Forwards,
                _ => Direction::Backwards,
            });

            self.translate_state.update(ctx.render_delta_time);
        }

        if ctx.mode != Mode::Sorts {
            return Ok(());
        }

        match action {
            AppAction::UserAction(UserAction::Up) => self.list.select_previous(),
            AppAction::UserAction(UserAction::Down) => self.list.select_next(),
            AppAction::UserAction(UserAction::Left) => self.sort_dir = SortDirection::Desc,
            AppAction::UserAction(UserAction::Right) => self.sort_dir = SortDirection::Asc,
            AppAction::UserAction(UserAction::Submit) if self.list.selected().is_some() => {
                action_tx.send(AppAction::SetSort(
                    self.list.selected().unwrap(),
                    self.sort_dir,
                ))?;
                action_tx.send(AppAction::UserAction(UserAction::SetMode(Mode::Home)))?;
                action_tx.send(AppAction::Search)?;
            }
            _ => {}
        }

        Ok(())
    }

    fn on_mouse(
        &mut self,
        ctx: &Context,
        event: &MouseEvent,
        action_tx: UnboundedSender<AppAction>,
    ) -> Result<()> {
        if ctx.mode != Mode::Sorts {
            return Ok(());
        }

        self.drag.on_mouse(event);

        if event.kind == MouseEventKind::Down(MouseButton::Left) {
            let pos = (event.column, event.row).into();

            // Close when clicking outside area
            if !self.drag.last_area().contains(pos) {
                action_tx.send(AppAction::UserAction(UserAction::SetMode(Mode::Home)))?;
            } else if self.drag.last_area().inner(Margin::new(1, 1)).contains(pos) {
                let area = self.drag.last_area();
                let rel_y = event.row - area.y - 1;
                self.list.select(Some(rel_y as usize));
            }
        }

        Ok(())
    }

    fn render(&mut self, ctx: &Context, frame: &mut Frame, area: Rect) -> Result<()> {
        let center = components::centered_rect(area, 50, 10).drag(&self.drag, area);

        let mut center_bottom = components::centered_rect(area, 50, 10);
        center_bottom.y = area.height + area.y;

        let vl = line::NORMAL.vertical_left;
        let vr = line::NORMAL.vertical_right;
        let sl = scrollbar::HORIZONTAL.begin;
        let sr = scrollbar::HORIZONTAL.end;

        let (order_left_col, order_right_col) = if self.sort_dir.is_desc() {
            (Color::Gray.to_rgb(), Color::White.to_rgb())
        } else {
            (Color::White.to_rgb(), Color::Gray.to_rgb())
        };
        let order_ind = [
            vl.into(),
            " ".into(),
            sl.fg(order_left_col),
            " ".into(),
            self.sort_dir.to_string().into(),
            " ".into(),
            sr.fg(order_right_col),
            " ".into(),
            vr.into(),
        ];

        let bg = Block::new()
            .bg(Color::Rgb(0, 36, 54))
            .borders(Borders::ALL)
            .title("Sorts")
            .title_top(Line::from_iter(order_ind).right_aligned());

        let items = ctx
            .source
            .source()
            .sorts()
            .into_iter()
            .enumerate()
            .map(|(i, f)| {
                format!(
                    "{}{f}",
                    if i == ctx.source_state.sort_idx {
                        "  " // TODO: Use user-defined char
                    } else {
                        "   "
                    }
                )
            });

        let list = List::new(items)
            .fg(Color::White.to_rgb())
            .highlight_style(Style::new().bg(Color::Rgb(40, 76, 94)))
            .block(bg);

        let translate = Translate::new(&self.translate_state, center_bottom.into(), center.into());
        let translate_area = Into::<Rect>::into(translate.area()).intersection(area);
        self.drag.set_last_area(translate_area);
        ClearOverlap.render(translate_area, frame.buffer_mut());
        translate.render_stateful_widget(list, area, frame.buffer_mut(), &mut self.list);

        Ok(())
    }
}
