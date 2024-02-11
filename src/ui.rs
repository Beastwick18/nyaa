use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use crate::{
    app::{App, Mode, Widgets},
    widget::{Popup, Widget},
};

pub static BORDER: BorderType = BorderType::Plain;
pub static DEFAULT_BLOCK: Block = Block::new().borders(Borders::ALL).border_type(BORDER);
pub static HI_BLOCK: Block = Block::new()
    .borders(Borders::ALL)
    .border_style(Style::new().fg(Color::LightCyan))
    .border_type(BORDER);

pub fn draw(widgets: &Widgets, app: &App, f: &mut Frame) {
    let chunks = Layout::new(
        Direction::Vertical,
        &[
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(1),
        ],
    )
    .split(f.size());

    let mode;
    match app.mode {
        Mode::Normal => {
            mode = "Normal";
        }
        Mode::Category => {
            mode = "Category";
            widgets.category.draw(f);
        }
        Mode::Sort => {
            mode = "Sort";
            widgets.sort.draw(f);
        }
        Mode::Search => {
            mode = "Editing";
        }
    }
    widgets.search.draw(f, app, chunks[1]);
    f.render_widget(Paragraph::new(format!("{}", mode)), chunks[0]);
}
