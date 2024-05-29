use std::ops::ControlFlow;

use crossterm::event::Event;
use ratatui::{layout::Rect, Frame};

use crate::app::Context;

use super::{notify_box::NotifyBox, Widget};

#[derive(Default)]
pub struct NotificationWidget {
    pub notifs: Vec<NotifyBox>,
}

impl NotificationWidget {
    pub fn is_animating(&self) -> bool {
        !self.notifs.is_empty()
    }

    pub fn add_notification(&mut self, notif: String) {
        let mut new_notif = NotifyBox::new(notif, 0.25);

        self.notifs.sort_unstable_by_key(|a| a.offset());
        let first_gap = self.notifs.iter().try_fold(0, |prev, x| {
            if x.offset() > prev {
                ControlFlow::Break((prev, x.offset()))
            } else {
                ControlFlow::Continue(x.offset() + x.height())
            }
        });
        let at_end = self
            .notifs
            .iter()
            .last()
            .map(|x| x.offset() + x.height())
            .unwrap_or(0);
        let offset = match first_gap {
            ControlFlow::Break((start, stop)) if stop >= new_notif.height() + start => start,
            _ => at_end,
        };
        new_notif.with_offset(offset);
        self.notifs.push(new_notif);
    }

    pub fn dismiss_all(&mut self) {
        self.notifs.iter_mut().for_each(|n| n.time = 1.0);
    }
}

impl Widget for NotificationWidget {
    fn draw(&mut self, f: &mut Frame, ctx: &Context, area: Rect) {
        for n in self.notifs.iter_mut() {
            n.draw(f, ctx, area);
        }
        self.notifs.retain(|n| !n.is_done());
    }

    fn handle_event(&mut self, _ctx: &mut Context, _e: &Event) {}

    fn get_help() -> Option<Vec<(&'static str, &'static str)>> {
        None
    }
}
