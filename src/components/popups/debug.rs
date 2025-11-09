use color_eyre::Result;
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::{
    layout::Rect,
    style::{Color, Stylize as _},
    widgets::{Block, Borders, Clear, Paragraph, Widget as _},
    Frame,
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    action::AppAction,
    app::Context,
    keys::{self, key_event_to_string, KeyComboStatus},
    mouse::drag::{DragExt, DragState},
    widgets::clear_overlap::ClearOverlap,
};

use super::Component;

#[derive(Default)]
pub struct Debug {
    possible_actions: Vec<String>,
    current_keycombo: String,
    current_keycombo_color: Color,
    drag: DragState,
}

impl Debug {
    pub fn boxed() -> Box<dyn Component> {
        Box::new(Self {
            drag: DragState::default(),
            ..Default::default()
        })
    }
}

impl Component for Debug {
    fn update(
        &mut self,
        ctx: &Context,
        _action: &AppAction,
        _action_tx: UnboundedSender<AppAction>,
    ) -> Result<()> {
        if ctx.keycombo.status() == &KeyComboStatus::Pending {
            self.possible_actions = ctx
                .config
                .keys
                .possible_actions(ctx.keycombo.events(), &ctx.mode, &ctx.input_mode)
                .map(|(keys, action)| {
                    format!(
                        "{} => {:?}",
                        keys.iter()
                            .map(key_event_to_string)
                            .collect::<Vec<String>>()
                            .join(""),
                        action
                    )
                })
                .collect();
        }
        let (mult, keycombo, keycombo_color) = (
            ctx.keycombo.repeat(),
            ctx.keycombo.events(),
            ctx.keycombo.status().color(),
        );

        self.current_keycombo = format!(
            "{}{}",
            mult.as_ref().map(ToString::to_string).unwrap_or_default(),
            keycombo
                .iter()
                .map(keys::key_event_to_string)
                .collect::<String>()
        );
        self.current_keycombo_color = keycombo_color;
        Ok(())
    }

    fn on_mouse(
        &mut self,
        ctx: &Context,
        event: &MouseEvent,
        _action_tx: UnboundedSender<AppAction>,
    ) -> Result<()> {
        if !ctx.show_debug {
            return Ok(());
        }

        self.drag.on_mouse(event);

        if let MouseEventKind::Down(MouseButton::Right) = event.kind {
            if self
                .drag
                .last_area()
                .contains((event.column, event.row).into())
            {
                self.drag.reset();
            }
        }

        Ok(())
    }

    fn render(&mut self, ctx: &Context, frame: &mut Frame, area: Rect) -> Result<()> {
        if !ctx.show_debug {
            return Ok(());
        }

        let center =
            Rect::new(area.right().saturating_sub(51), area.y + 1, 50, 25).drag(&self.drag, area);

        ClearOverlap.render(center, frame.buffer_mut());
        Clear.render(center, frame.buffer_mut());

        let bg = Block::new()
            .bg(Color::Rgb(0, 36, 54))
            .borders(Borders::ALL)
            .title("Debug");
        let x = self
            .current_keycombo
            .clone()
            .fg(self.current_keycombo_color);
        let p = Paragraph::new(format!(
            "rdt: {}\n\nMode: {}\n\nPossible Actions:\n{}\n\nKeycombo: {}",
            ctx.render_delta_time,
            ctx.mode,
            self.possible_actions.join("\n"),
            x
        ))
        .block(bg);

        self.drag.set_last_area(center);
        frame.render_widget(p, center);

        Ok(())
    }
}
