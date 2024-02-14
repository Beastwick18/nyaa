use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::Stylize,
    widgets::Paragraph,
    Frame,
};

use crate::{
    app::{App, Mode, Widgets},
    widget::{centered_rect, Popup, Widget},
};

pub fn draw(widgets: &mut Widgets, app: &App, f: &mut Frame) {
    let layout = Layout::new(
        Direction::Vertical,
        &[
            Constraint::Length(match app.show_hints {
                true => 1,
                false => 0,
            }), // TODO: Maybe remove this, keys are obvious. Or make hiding it a config option
            Constraint::Length(3),
            Constraint::Min(1),
        ],
    )
    .split(f.size());

    widgets.search.draw(f, app, layout[1]);
    widgets.results.draw(f, app, layout[2]);
    let mode;
    match app.mode {
        Mode::Normal => {
            mode = "Normal";
        }
        Mode::Category => {
            mode = "Category";
            widgets.category.draw(f, &app.theme);
        }
        Mode::Sort => {
            mode = "Sort";
            widgets.sort.draw(f, &app.theme);
        }
        Mode::Search => {
            mode = "Search";
        }
        Mode::Filter => {
            mode = "Filter";
            widgets.filter.draw(f, &app.theme);
        }
        Mode::Theme => {
            mode = "Theme";
            widgets.theme.draw(f, &app.theme);
        }
        Mode::Loading => {
            mode = "Loading";
            let area = centered_rect(10, 1, f.size());
            widgets.results.clear();
            widgets.results.draw(f, app, layout[2]);
            f.render_widget(Paragraph::new("Loading..."), area);
        }
    }
    f.render_widget(
        Paragraph::new(mode)
            .bg(app.theme.bg)
            .fg(app.theme.border_focused_color),
        layout[0],
    ); // TODO: Debug only
}
