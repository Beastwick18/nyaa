use color_eyre::Result;
use ratatui::{
    layout::Rect,
    style::{Color, Stylize as _},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::{
    action::AppAction,
    app::Context,
    keys::{self, key_event_to_string, KeyComboStatus},
};

use super::Component;

pub struct ActionsComponent {
    actions: Vec<String>,
    possible_actions: Vec<String>,
    current_keycombo: String,
    current_keycombo_color: Color,
}

impl ActionsComponent {
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
            possible_actions: Vec::new(),
            current_keycombo: String::new(),
            current_keycombo_color: Color::White,
        }
    }
}

impl Component for ActionsComponent {
    fn update(
        &mut self,
        ctx: &Context,
        _action: &AppAction,
    ) -> Result<Option<crate::action::AppAction>> {
        if let Some(keymap) = ctx.config.keys.get(&ctx.mode) {
            self.actions = keymap
                .iter()
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

            if ctx.keycombo.status() == &KeyComboStatus::Pending {
                self.possible_actions = keymap
                    .iter()
                    .filter(|(keys, _action)| keys.starts_with(ctx.keycombo.events()))
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
        Ok(None)
    }

    fn render(&mut self, ctx: &Context, frame: &mut Frame, area: Rect) -> Result<()> {
        let x = self
            .current_keycombo
            .clone()
            .fg(self.current_keycombo_color);
        let p = Paragraph::new(format!(
            "rdt: {}\n\nMode: {}\n\nMappings:\n{}\n\nPossible Actions:\n{}\n\nKeycombo: {}",
            ctx.render_delta_time,
            ctx.mode,
            self.actions.join("\n"),
            self.possible_actions.join("\n"),
            x
        ))
        .block(
            Block::new()
                .fg(self.current_keycombo_color)
                .borders(Borders::ALL)
                .title("User Actions"),
        );
        frame.render_widget(p, area);

        Ok(())
    }
}
