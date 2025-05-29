use color_eyre::Result;
use ratatui::{
    layout::Rect,
    style::{Color, Stylize as _},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::{
    action::{AppAction, UserAction},
    animate::{translate::Translate, Animation, AnimationState, FloatRect, Smoothing},
    app::Context,
    color::to_rgb,
};

use super::Component;

#[derive(Clone, Copy)]
pub enum NotificationType {
    Error,
    Warning,
    Info,
}

#[derive(Default)]
pub struct NotificationContainer {
    notifications: Vec<Notification>,
}

impl NotificationContainer {
    pub fn boxed() -> Box<dyn Component> {
        Box::new(Self::default())
    }

    fn add_notification(&mut self, notification: Notification) {
        for n in self.notifications.iter_mut() {
            n.add_offset(notification.height() as i16);
        }
        self.notifications.push(notification);
    }
}

impl Component for NotificationContainer {
    fn update(&mut self, ctx: &Context, action: &AppAction) -> Result<Option<AppAction>> {
        // Create notifications from actions
        match action {
            AppAction::Error(err) | AppAction::UserAction(UserAction::NotifyError(err)) => {
                self.add_notification(Notification::new(err, true, NotificationType::Error));
            }
            AppAction::Warning(warn) | AppAction::UserAction(UserAction::NotifyWarning(warn)) => {
                self.add_notification(Notification::new(warn, false, NotificationType::Warning));
            }
            AppAction::Info(info) | AppAction::UserAction(UserAction::NotifyInfo(info)) => {
                self.add_notification(Notification::new(info, false, NotificationType::Info));
            }
            AppAction::UserAction(UserAction::ClearNotifications) => {
                for n in self.notifications.iter_mut() {
                    n.dismiss();
                }
            }
            _ => {}
        }

        // Separate out all finished notifications
        let done: Vec<_> = self
            .notifications
            .iter()
            .filter(|n| n.is_done())
            .cloned()
            .collect();

        // Clear all finished notifications
        self.notifications.retain(|n| !n.is_done());

        // Remove offset from notifications when one is dismissed
        for n in self.notifications.iter_mut() {
            for d in done.iter() {
                if n.offset() > d.offset() {
                    n.add_offset(-(d.height() as i16));
                }
            }
        }

        for n in self.notifications.iter_mut() {
            n.update(ctx, action)?;
        }
        Ok(None)
    }

    fn render(&mut self, ctx: &Context, frame: &mut Frame, area: Rect) -> Result<()> {
        for n in self.notifications.iter_mut() {
            n.render(ctx, frame, area)?;
        }
        Ok(())
    }
}

#[derive(Clone)]
struct Notification {
    enter_state: AnimationState,
    waiting_state: AnimationState,
    exit_state: AnimationState,
    prev_offset: u16,
    offset: u16,
    between_state: AnimationState,
    width: u16,
    height: u16,
    lines: Vec<String>,
    persist: bool,
    notif_type: NotificationType,
}

impl Notification {
    fn new(
        message: impl Into<String>,
        persist: bool,
        notif_type: NotificationType,
    ) -> Notification {
        let (width, height, lines) = Self::breakup_text(&message.into());
        Self {
            enter_state: AnimationState::from_secs(0.15)
                .playing(true)
                .forwards()
                .smoothing(Smoothing::EaseOut),
            waiting_state: AnimationState::from_secs(5.0)
                .playing(true)
                .forwards()
                .smoothing(Smoothing::Linear),
            exit_state: AnimationState::from_secs(0.1)
                .playing(true)
                .forwards()
                .smoothing(Smoothing::EaseIn),
            between_state: AnimationState::from_secs(0.15)
                .playing(true)
                .forwards()
                .smoothing(Smoothing::EaseOut)
                .ending(),
            prev_offset: 0,
            offset: 0,
            width,
            height,
            lines,
            persist,
            notif_type,
        }
    }

    fn breakup_text(msg: &str) -> (u16, u16, Vec<String>) {
        let max_width = 32;
        let lines: Vec<String> = textwrap::wrap(msg, max_width)
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        let width = lines.iter().map(String::len).max().unwrap_or(1) as u16 + 2;
        let height = lines.len() as u16 + 2;
        (width, height, lines)
    }

    fn is_done(&self) -> bool {
        self.exit_state.is_done()
    }

    fn offset(&self) -> u16 {
        self.offset
    }

    fn add_offset(&mut self, offset: i16) {
        // Don't move offset if exiting
        if self.waiting_state.is_done() {
            return;
        }

        self.enter_state.goto_end();
        self.prev_offset = self.offset;
        self.offset = self.offset.saturating_add_signed(offset);
        self.between_state.goto_start();
    }

    fn height(&self) -> u16 {
        self.height
    }

    fn dismiss(&mut self) {
        self.persist = false;

        self.enter_state.goto_end();
        self.between_state.goto_end();
        self.waiting_state.goto_end();
    }
}

impl Component for Notification {
    fn update(&mut self, ctx: &Context, action: &AppAction) -> Result<Option<AppAction>> {
        if action == &AppAction::Render {
            if self.persist {
                self.enter_state
                    .then(&mut self.between_state)
                    .update(ctx.render_delta_time);
            } else {
                self.enter_state
                    .then(&mut self.between_state)
                    .then(&mut self.waiting_state)
                    .then(&mut self.exit_state)
                    .update(ctx.render_delta_time);

                if !self.between_state.is_done() && !self.persist {
                    self.waiting_state.update(ctx.render_delta_time);
                }
            };
        }

        Ok(None)
    }

    fn render(&mut self, _ctx: &Context, frame: &mut Frame, area: Rect) -> Result<()> {
        // TODO: Add with themes
        let color = match self.notif_type {
            NotificationType::Error => to_rgb(Color::Red),
            NotificationType::Warning => to_rgb(Color::Yellow),
            NotificationType::Info => to_rgb(Color::Cyan),
        };

        let bg = Block::new()
            .fg(color)
            .bg(Color::Rgb(34, 36, 54))
            .borders(Borders::ALL);
        let p = Paragraph::new(self.lines.join("\n"))
            .fg(Color::White)
            .block(bg);

        let width = self.width as f64;
        let height = self.height as f64;

        let (x, y) = (area.right() as f64 - 1.0 - width, area.top() as f64 + 1.0);

        let enter = Translate::new(
            &self.enter_state,
            FloatRect::new(x, y - height, width, height),
            FloatRect::new(x, y, width, height),
        );
        let between = Translate::new(
            &self.between_state,
            FloatRect::new(x, y + self.prev_offset as f64, width, height),
            FloatRect::new(x, y + self.offset as f64, width, height),
        );
        let wait = Translate::new(
            &self.waiting_state,
            FloatRect::new(x, y + self.offset as f64, width, height),
            FloatRect::new(x, y + self.offset as f64, width, height),
        );
        let exit = Translate::new(
            &self.exit_state,
            FloatRect::new(x, y + self.offset as f64, width, height),
            FloatRect::new(x + width, y + self.offset as f64, width, height),
        );

        let translate = if self.persist {
            enter.then(&between)
        } else {
            enter.then(&between).then(&wait).then(&exit)
        };
        translate.render_widget(p, area, frame.buffer_mut());

        Ok(())
    }
}
