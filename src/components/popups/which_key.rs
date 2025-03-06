use std::time::Duration;

use color_eyre::Result;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Stylize as _},
    symbols,
    text::Line,
    widgets::{Block, Clear, List, Widget as _},
    Frame,
};

use crate::{
    action::{AppAction, UserAction},
    animate::{translate::Translate, Animation, AnimationState, Direction, Smoothing},
    app::Context,
    components::borders,
    keys::{key_event_to_string, KeyComboStatus},
    widgets::clear_overlap::ClearOverlap,
};

use super::Component;

pub struct WhichKeyComponent {
    translate: Translate,
    current_keycombo: String,
    possible_actions: Vec<(String, String)>,
    wait_duration: Duration,
    current_time: Duration,
}

impl WhichKeyComponent {
    pub fn new() -> Box<WhichKeyComponent> {
        let wait_duration = Duration::from_secs(1);
        Box::new(Self {
            translate: AnimationState::new(0.12)
                .playing(true)
                .backwards()
                .smoothing(Smoothing::EaseInAndOut)
                .into(),
            current_keycombo: String::new(),
            possible_actions: Vec::new(),
            wait_duration,
            current_time: wait_duration,
        })
    }
}

impl Component for WhichKeyComponent {
    fn update(&mut self, ctx: &Context, action: &AppAction) -> Result<Option<AppAction>> {
        if let AppAction::UserAction(UserAction::WhichKey) = action {
            self.current_time = Duration::ZERO;
        }
        if let AppAction::Tick = action {
            self.translate
                .state_mut()
                .set_direction(match self.current_time.is_zero() {
                    true => Direction::Forwards,
                    false => Direction::Backwards,
                });
            self.translate.state_mut().update();

            if let Some(keymap) = ctx.config.keys.get(&ctx.mode) {
                let events = if ctx.keycombo.status() == &KeyComboStatus::Pending {
                    // Decrement time till WhichKey popup appears
                    self.current_time = self.current_time.saturating_sub(Duration::from_millis(16));
                    Some(ctx.keycombo.events())
                } else {
                    None
                };

                self.current_keycombo = events
                    .map(|e| e.iter().map(key_event_to_string).collect::<String>())
                    .unwrap_or_default();

                self.possible_actions = keymap
                    .into_iter()
                    .filter(|(keys, _action)| events.is_none() || keys.starts_with(events.unwrap()))
                    .map(|(keys, action)| {
                        (
                            keys.iter()
                                .map(key_event_to_string)
                                .skip(self.current_keycombo.len())
                                .collect::<String>(),
                            action.to_string(),
                        )
                    })
                    .collect();
            }
        }

        Ok(None)
    }

    fn on_key(&mut self, ctx: &Context, _key: &crossterm::event::KeyEvent) -> Result<()> {
        if ctx.keycombo.status() != &KeyComboStatus::Pending {
            self.current_time = self.wait_duration;
        }
        Ok(())
    }

    fn render(&mut self, _ctx: &Context, frame: &mut Frame, area: Rect) -> Result<()> {
        let width = 36;
        let height = 25;
        let right = Rect {
            x: area.width.saturating_sub(width + 1),
            y: area.height.saturating_sub(height + 1),
            width,
            height,
        };
        let bottom_right = Rect {
            x: area.width.saturating_sub(width + 1),
            y: area.height,
            width,
            height,
        };

        let t_area = self.translate.start(bottom_right).stop(right).area();

        let area = t_area.intersection(area);
        ClearOverlap.render(area, frame.buffer_mut());
        Clear.render(area, frame.buffer_mut());

        let block = Block::new()
            .bg(Color::Rgb(34, 36, 54))
            .borders(borders(t_area, area))
            .border_set(symbols::border::ROUNDED)
            .title(" Possible Actions ")
            .title_alignment(Alignment::Center);
        let list = List::new(self.possible_actions.iter().map(|(rest, action)| {
            Line::from_iter([
                self.current_keycombo.as_str().fg(Color::Cyan),
                rest.into(),
                " ➜ ".fg(Color::DarkGray),
                action.as_str().fg(Color::White),
            ])
        }))
        .fg(Color::White)
        .block(block);
        list.render(area, frame.buffer_mut());

        Ok(())
    }
}
