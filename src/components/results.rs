use color_eyre::Result;
use ratatui::{
    layout::{Margin, Rect},
    style::{Color, Stylize as _},
    symbols::line,
    text::Line,
    widgets::{Block, Borders, Paragraph, Table, TableState},
    Frame,
};

use crate::{
    action::{AppAction, TaskAction},
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
            }
            AppAction::Search(_) => {
                self.results = None;
            }
            _ => {}
        }

        let (keycombo, keycombo_color) = if ctx.keycombo.is_empty() && ctx.last_keycombo.is_some() {
            let keycombo = ctx.last_keycombo.as_ref().unwrap();
            (keycombo.inner(), keycombo.color())
        } else {
            (&ctx.keycombo, Color::White)
        };
        self.current_keycombo = keycombo.iter().map(keys::key_event_to_string).collect();
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
                .clone()
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
            let table: Table = results.table.clone().into();
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
