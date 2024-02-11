use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
    Terminal,
};

use crate::widget::{category::CategoryPopup, Popup};

static BORDER: BorderType = BorderType::Plain;

pub enum Mode {
    Normal,
    Category,
}

pub struct App {
    pub mode: Mode,
}

impl Default for App {
    fn default() -> Self {
        App { mode: Mode::Normal }
    }
}

fn normal_events(app: &mut App, e: &Event) {
    if let Event::Key(KeyEvent {
        code,
        kind: KeyEventKind::Press,
        ..
    }) = e
    {
        match code {
            KeyCode::Char('c') => {
                app.mode = Mode::Category;
            }
            _ => {}
        }
    }
}

pub async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    let mut cat = CategoryPopup::default();
    loop {
        terminal.draw(|f| {
            let chunks = Layout::new(
                Direction::Vertical,
                &[
                    Constraint::Length(1),
                    Constraint::Length(3),
                    Constraint::Min(1),
                ],
            )
            .split(f.size());

            let def_block = Block::new().borders(Borders::ALL).border_type(BORDER);

            let hi_block = Block::new()
                .borders(Borders::ALL)
                .border_style(Style::new().fg(Color::LightCyan))
                .border_type(BORDER);

            match app.mode {
                Mode::Normal => {
                    f.render_widget(Paragraph::new(format!("{}", cat.category)), chunks[0]);
                }
                Mode::Category => {
                    cat.draw(f);
                }
            }
        })?;

        let evt = event::read()?;
        match app.mode {
            Mode::Category => {
                cat.handle_event(&mut app, &evt);
            }
            Mode::Normal => {
                normal_events(&mut app, &evt);
            }
        }

        if let Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            ..
        }) = evt
        {
            match code {
                KeyCode::Char('q') => return Ok(()),
                _ => {}
            }
        }
    }
}
