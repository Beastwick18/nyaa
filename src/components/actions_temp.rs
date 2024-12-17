use color_eyre::Result;
use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::{
    action::AppAction,
    app::Context,
    keys::{self, key_event_to_string},
};

use super::Component;

pub struct ActionsComponent {
    actions: Vec<String>,
    current_keycombo: String,
}

impl ActionsComponent {
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
            current_keycombo: String::new(),
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
        }
        let keycombo = if ctx.keycombo.is_empty() {
            &ctx.last_successful_keycombo
        } else {
            &ctx.keycombo
        };
        self.current_keycombo = keycombo.iter().map(keys::key_event_to_string).collect();
        Ok(None)
    }

    fn render(&mut self, ctx: &Context, frame: &mut Frame, area: Rect) -> Result<()> {
        let p = Paragraph::new(format!(
            "Mode: {}\nKeys:\n{}\nKeycombo: {}",
            ctx.mode,
            self.actions.join("\n"),
            self.current_keycombo
        ))
        .block(Block::new().borders(Borders::ALL).title("User Actions"));
        frame.render_widget(p, area);

        Ok(())
    }
}
