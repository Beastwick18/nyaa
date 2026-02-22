use color_eyre::Result;
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::{
    Frame,
    layout::{Margin, Rect},
    style::{Color, Style, Stylize as _},
    widgets::{Block, Borders, List, ListState, Widget as _},
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    action::{AppAction, UserAction},
    animate::{Animation, AnimationState, Direction, Smoothing, translate::Translate},
    app::{Context, Mode},
    color::ColorRgbExt,
    components,
    mouse::drag::{DragEdge, DragExt, DragState},
    sources::SourceTask,
    widgets::clear_overlap::ClearOverlap,
};

use super::Component;

pub struct Filters {
    translate_state: AnimationState,
    drag: DragState,
    list: ListState,
}

impl Filters {
    pub fn boxed() -> Box<dyn Component> {
        Box::new(Self {
            translate_state: AnimationState::new(6.0)
                .playing(true)
                .backwards()
                .smoothing(Smoothing::EaseInAndOut),
            drag: DragState::edge(DragEdge::ALL),
            list: ListState::default().with_selected(Some(0)),
        })
    }
}

impl Component for Filters {
    fn update(
        &mut self,
        ctx: &Context,
        action: &AppAction,
        action_tx: UnboundedSender<AppAction>,
    ) -> Result<()> {
        match action {
            AppAction::Render => {
                self.translate_state.set_direction(match ctx.mode {
                    Mode::Filters => Direction::Forwards,
                    _ => Direction::Backwards,
                });

                self.translate_state.update(ctx.render_delta_time);
            }
            AppAction::UserAction(UserAction::Up) => self.list.select_previous(),
            AppAction::UserAction(UserAction::Down) => self.list.select_next(),
            AppAction::UserAction(UserAction::Submit) if self.list.selected().is_some() => {
                action_tx.send(AppAction::SetFilter(self.list.selected().unwrap()))?;
                action_tx.send(AppAction::UserAction(UserAction::SetMode(Mode::Home)))?;
                action_tx.send(AppAction::Search(String::new()))?;
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
        if ctx.mode != Mode::Filters {
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

        ClearOverlap.render(center, frame.buffer_mut());

        let bg = Block::new().bg(Color::Rgb(0, 36, 54)).borders(Borders::ALL);

        let items = ctx
            .source
            .source()
            .filters()
            .into_iter()
            .enumerate()
            .map(|(i, f)| {
                format!(
                    "{} {f}",
                    if i == ctx.source_state.filter_idx {
                        "*"
                    } else {
                        " "
                    }
                )
            });

        let list = List::new(items)
            .fg(Color::White.to_rgb())
            .highlight_style(Style::new().bg(Color::Rgb(40, 76, 94)))
            .block(bg);

        let translate = Translate::new(&self.translate_state, center_bottom.into(), center.into());
        self.drag.set_last_area(translate.area().into());
        translate.render_stateful_widget(list, area, frame.buffer_mut(), &mut self.list);

        Ok(())
    }
}
