use std::fmt::Display;

use crossterm::event::Event;
use ratatui::{layout::Rect, Frame};
use serde::{Deserialize, Serialize};

use crate::app::Context;

use super::{notify_box::NotifyBox, Corner, Widget};

static MAX_NOTIFS: usize = 100;

#[derive(Clone, Copy, PartialEq)]
pub enum NotificationType {
    Info,
    Warning,
    Error,
    Success,
}

#[derive(Clone)]
pub struct Notification {
    pub content: String,
    pub notif_type: NotificationType,
}

impl Notification {
    pub fn info<S: Display>(content: S) -> Self {
        Self {
            content: content.to_string(),
            notif_type: NotificationType::Info,
        }
    }

    pub fn warning<S: Display>(content: S) -> Self {
        Self {
            content: content.to_string(),
            notif_type: NotificationType::Warning,
        }
    }

    pub fn error<S: Display>(content: S) -> Self {
        Self {
            content: content.to_string(),
            notif_type: NotificationType::Error,
        }
    }

    pub fn success<S: Display>(content: S) -> Self {
        Self {
            content: content.to_string(),
            notif_type: NotificationType::Success,
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct NotificationConfig {
    pub position: Option<Corner>,
    pub duration: Option<f64>,
    pub max_width: Option<u16>,
    pub animation_speed: Option<f64>,
}

pub struct NotificationWidget {
    notifs: Vec<NotifyBox>,
    duration: f64,
    position: Corner,
    max_width: u16,
    animation_speed: f64,
}

impl Default for NotificationWidget {
    fn default() -> Self {
        Self {
            notifs: vec![],
            duration: 3.0,
            position: Corner::TopRight,
            max_width: 75,
            animation_speed: 4.,
        }
    }
}

impl NotificationWidget {
    pub fn load_config(&mut self, conf: &NotificationConfig) {
        self.position = conf.position.unwrap_or(self.position);
        self.duration = conf.duration.unwrap_or(self.duration).max(0.01);
        self.max_width = conf.max_width.unwrap_or(self.max_width);
        self.animation_speed = conf.animation_speed.unwrap_or(self.animation_speed);
    }

    pub fn is_animating(&self) -> bool {
        !self.notifs.is_empty()
    }

    pub fn add(&mut self, notif: Notification) {
        let persist = matches!(notif.notif_type, NotificationType::Error);
        let notif = NotifyBox::new(
            notif,
            self.duration,
            self.position,
            self.animation_speed,
            self.max_width,
            persist,
        );

        self.notifs
            .iter_mut()
            .for_each(|n| n.add_offset(notif.height() as i32));

        self.dismiss_oldest();

        self.notifs.push(notif);
    }

    pub fn dismiss_all(&mut self) {
        self.notifs.iter_mut().for_each(|n| n.time = 1.0);
    }

    fn dismiss_oldest(&mut self) {
        if self.notifs.len() >= MAX_NOTIFS {
            self.notifs
                .drain(..=self.notifs.len().saturating_sub(MAX_NOTIFS));
        }
    }

    pub fn update(&mut self, deltatime: f64, area: Rect) -> bool {
        let res = self
            .notifs
            .iter_mut()
            .fold(false, |acc, x| x.update(deltatime, area) || acc);
        let finished = self
            .notifs
            .iter()
            .filter_map(|n| match n.is_done() {
                true => Some((n.offset(), n.height())),
                false => None,
            })
            .collect::<Vec<(u16, u16)>>();
        // Offset unfinished notifications by gap left from finished notifs
        for (offset, height) in finished.iter() {
            self.notifs.iter_mut().for_each(|n| {
                if n.get_type() == NotificationType::Error && n.offset() > *offset {
                    n.add_offset(-(*height as i32));
                }
            })
        }
        // Delete finished notifications
        self.notifs.retain(|n| !n.is_done());
        res
    }
}

impl Widget for NotificationWidget {
    fn draw(&mut self, f: &mut Frame, ctx: &Context, area: Rect) {
        self.notifs.iter_mut().for_each(|n| n.draw(f, ctx, area));
    }

    fn handle_event(&mut self, _ctx: &mut Context, _e: &Event) {}

    fn get_help() -> Option<Vec<(&'static str, &'static str)>> {
        None
    }
}
