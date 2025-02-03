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
    keys::{self, key_event_to_string, KeyCombo},
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

            self.possible_actions = keymap
                .iter()
                .filter(|(keys, _action)| keys.starts_with(&ctx.keycombo))
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
        let (keycombo, keycombo_color) = if ctx.keycombo.is_empty() && ctx.last_keycombo.is_some() {
            match ctx.last_keycombo.as_ref().unwrap() {
                KeyCombo::Successful(vec) => (vec, Color::Cyan),
                KeyCombo::Cancelled(vec) => (vec, Color::Magenta),
                KeyCombo::Unmatched(vec) => (vec, Color::Red),
            }
        } else {
            (&ctx.keycombo, Color::White)
        };
        self.current_keycombo = keycombo.iter().map(keys::key_event_to_string).collect();
        self.current_keycombo_color = keycombo_color;
        Ok(None)
    }

    fn render(&mut self, ctx: &Context, frame: &mut Frame, area: Rect) -> Result<()> {
        let x = self
            .current_keycombo
            .clone()
            .fg(self.current_keycombo_color);
        let p = Paragraph::new(format!(
            "Mode: {}\n\nMappings:\n{}\n\nPossible Actions:\n{}\n\nKeycombo: {}",
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
