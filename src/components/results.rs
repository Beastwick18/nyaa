use color_eyre::Result;
use ratatui::{
    layout::{Margin, Rect},
    style::{Color, Style, Stylize as _},
    symbols::line,
    text::Line,
    widgets::{Block, Borders, Paragraph, Table, TableState},
    Frame,
};

use crate::{
    action::{AppAction, TaskAction, UserAction},
    app::Context,
    keys,
    result::Results,
};

use super::Component;

pub struct ResultsComponent {
    results: Option<Results>,
    table_state: TableState,
    current_keycombo: String,
    current_keycombo_color: Color,
}

impl ResultsComponent {
    pub fn new() -> Self {
        Self {
            results: None,
            table_state: TableState::default(),
            current_keycombo: String::new(),
            current_keycombo_color: Color::White,
        }
    }
}

impl Component for ResultsComponent {
    fn update(
        &mut self,
        ctx: &Context,
        action: &AppAction,
    ) -> color_eyre::eyre::Result<Option<AppAction>> {
        match action {
            AppAction::Task(TaskAction::SourceResults(results)) => {
                self.results.clone_from(results);
                self.table_state.select_first();
            }
            AppAction::Search(_) => {
                self.results = None;
            }
            AppAction::UserAction(UserAction::Up) => {
                self.table_state.select_previous();
            }
            AppAction::UserAction(UserAction::Down) => {
                self.table_state.select_next();
            }
            AppAction::UserAction(UserAction::Top) => {
                self.table_state.select_first();
            }
            AppAction::UserAction(UserAction::Bottom) => {
                self.table_state.select_last();
            }
            _ => {}
        }

        let (mult, keycombo, keycombo_color) = (
            ctx.keycombo
                .repeat()
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default(),
            ctx.keycombo
                .events()
                .iter()
                .map(keys::key_event_to_string)
                .collect::<String>(),
            ctx.keycombo.status().color(),
        );

        self.current_keycombo = format!("{mult}{keycombo}");
        self.current_keycombo_color = keycombo_color;

        Ok(None)
    }

    fn render(&mut self, _ctx: &Context, frame: &mut Frame, area: Rect) -> Result<()> {
        let mut block = Block::new()
            .fg(Color::Rgb(255, 255, 255))
            .borders(Borders::ALL);
        if !self.current_keycombo.is_empty() {
            let combo = self
                .current_keycombo
                .as_str()
                .fg(self.current_keycombo_color);
            let vr = line::NORMAL.vertical_right;
            let vl = line::NORMAL.vertical_left;
            // let vr = '';
            // let vl = '';
            let keycombo = Line::from_iter([
                format!("{} ", vl).fg(Color::Rgb(255, 255, 255)),
                combo,
                format!(" {}", vr).fg(Color::Rgb(255, 255, 255)),
            ]);

            block = block.title_bottom(keycombo.right_aligned());
        };

        frame.render_widget(block, area);

        let area = area.inner(Margin::new(1, 1));
        if let Some(results) = &self.results {
            if results.items.is_empty() {
                let text = "No results";
                let paragraph = Paragraph::new(text);
                let center = super::centered_rect(area, text.len() as u16, 1);
                frame.render_widget(paragraph, center);
            }
            let table: Table = (&results.table).into();
            let table = table.row_highlight_style(Style::new().bg(Color::Rgb(58, 61, 92)));
            frame.render_stateful_widget(table, area, &mut self.table_state);
        } else {
            let text = "Loading...";
            let paragraph = Paragraph::new(text);
            let center = super::centered_rect(area, text.len() as u16, 1);
            frame.render_widget(paragraph, center);
        }

        Ok(())
    }
}
