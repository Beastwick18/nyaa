use crossterm::event::Event;
use ratatui::{layout::Rect, Frame};
use serde::{Deserialize, Serialize};

use crate::app::Context;

use super::{
    notify_box::{NotifyBox, NotifyPosition},
    Widget,
};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct NotificationConfig {
    pub position: Option<NotifyPosition>,
    pub duration: Option<f64>,
}

pub struct NotificationWidget {
    notifs: Vec<NotifyBox>,
    duration: f64,
    position: NotifyPosition,
}

impl Default for NotificationWidget {
    fn default() -> Self {
        Self {
            notifs: vec![],
            duration: 3.0,
            position: NotifyPosition::TopRight,
        }
    }
}

impl NotificationWidget {
    pub fn load_config(&mut self, conf: &NotificationConfig) {
        self.position = conf.position.unwrap_or(self.position);
        self.duration = conf.duration.unwrap_or(self.duration).max(0.01);
    }

    pub fn is_animating(&self) -> bool {
        !self.notifs.is_empty()
    }

    pub fn add_notification(&mut self, notif: String) {
        let new_notif = NotifyBox::new(notif, self.duration, self.position, false);

        self.notifs
            .iter_mut()
            .for_each(|n| n.add_offset(new_notif.height()));

        self.notifs.push(new_notif);
    }

    pub fn add_error(&mut self, error: String) {
        let new_notif = NotifyBox::new(error, 0.0, self.position, true);

        self.notifs
            .iter_mut()
            .for_each(|n| n.add_offset(new_notif.height()));

        self.notifs.push(new_notif);
    }

    pub fn dismiss_all(&mut self) {
        self.notifs.iter_mut().for_each(|n| n.time = 1.0);
    }

    pub fn update(&mut self, deltatime: f64, area: Rect) -> bool {
        self.notifs.iter_mut().fold(false, |acc, x| {
            let res = x.update(deltatime, area);
            x.is_done() || res || acc
        })
    }
}

impl Widget for NotificationWidget {
    fn draw(&mut self, f: &mut Frame, ctx: &Context, area: Rect) {
        let res = self
            .notifs
            .iter()
            .filter_map(|n| match n.is_done() {
                true => Some((n.offset(), n.height())),
                false => None,
            })
            .collect::<Vec<(u16, u16)>>();
        for (offset, height) in res.iter() {
            self.notifs.iter_mut().for_each(|n| {
                if n.is_error() && n.offset() > *offset {
                    n.add_offset(-(*height as i32));
                }
            })
        }
        self.notifs.retain(|n| !n.is_done());

        for n in self.notifs.iter_mut() {
            n.draw(f, ctx, area);
        }
    }

    fn handle_event(&mut self, _ctx: &mut Context, _e: &Event) {}

    fn get_help() -> Option<Vec<(&'static str, &'static str)>> {
        None
    }
}
