use color_eyre::Result;
use crossterm::event::KeyEvent;
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
    animate::{translate::Translate, AnimationState, Direction, Smoothing},
    app::Context,
    components::borders,
    keys::{key_event_to_string, KeyComboStatus},
    widgets::clear_overlap::ClearOverlap,
};

use super::Component;

pub struct WhichKeyComponent {
    translate: AnimationState,
    wait_state: AnimationState,
    current_keycombo: String,
    possible_actions: Vec<(String, String)>,
}

impl WhichKeyComponent {
    pub fn boxed() -> Box<dyn Component> {
        Box::new(Self {
            translate: AnimationState::new(10.0)
                .playing(true)
                .backwards()
                .smoothing(Smoothing::EaseInAndOut),
            wait_state: AnimationState::from_secs(1.0).playing(true).backwards(),
            current_keycombo: String::new(),
            possible_actions: Vec::new(),
        })
    }
}

impl Component for WhichKeyComponent {
    fn update(&mut self, ctx: &Context, action: &AppAction) -> Result<Option<AppAction>> {
        if action == &AppAction::UserAction(UserAction::WhichKey) {
            self.wait_state.set_direction(Direction::Forwards);
            self.wait_state.goto_end();
        } else if action == &AppAction::Render {
            self.wait_state.update(ctx.render_delta_time);
            if self.wait_state.is_ending() {
                self.translate.set_direction(Direction::Forwards);
            }
            self.translate.update(ctx.render_delta_time);

            let possible_actions = if ctx.keycombo.status() == &KeyComboStatus::Pending {
                self.current_keycombo = ctx
                    .keycombo
                    .events()
                    .iter()
                    .map(key_event_to_string)
                    .collect::<String>();

                ctx.config
                    .keys
                    .possible_actions(ctx.keycombo.events(), &ctx.mode, &ctx.input_mode)
            } else {
                self.current_keycombo.clear();

                ctx.config
                    .keys
                    .possible_actions(&[], &ctx.mode, &ctx.input_mode)
            };

            self.possible_actions = possible_actions
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

        Ok(None)
    }

    fn on_key(&mut self, ctx: &Context, _key: &KeyEvent) -> Result<()> {
        if ctx.keycombo.status() != &KeyComboStatus::Pending {
            self.wait_state.goto_start();
            self.wait_state.set_direction(Direction::Backwards);
            self.translate.set_direction(Direction::Backwards);
        } else {
            self.wait_state.set_direction(Direction::Forwards);
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

        let t_area: Rect = Translate::new(&self.translate, bottom_right.into(), right.into())
            .area()
            .into();

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
