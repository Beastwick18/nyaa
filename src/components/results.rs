use color_eyre::Result;
use ratatui::{
    layout::{Margin, Rect},
    style::{Color, Style, Stylize as _},
    symbols::line,
    text::Line,
    widgets::{Block, Borders, Paragraph, Table, TableState},
    Frame,
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    action::{AppAction, TaskAction, UserAction},
    animate::AnimationState,
    app::Context,
    color::ColorRgbExt,
    keys,
    result::Results,
};

use super::Component;

const LOADING_CHARS: [char; 5] = ['⠾', '⠽', '⠻', '⠯', '⠷'];

pub struct ResultsComponent {
    results: Option<Results>,
    table_state: TableState,
    current_keycombo: String,
    current_keycombo_color: Color,
    loading_state: AnimationState,
}

impl ResultsComponent {
    pub fn new() -> Self {
        Self {
            results: None,
            table_state: TableState::default(),
            current_keycombo: String::new(),
            current_keycombo_color: Color::Rgb(255, 255, 255),
            loading_state: AnimationState::new(1.0).playing(true).forwards(),
        }
    }
}

impl Component for ResultsComponent {
    fn update(
        &mut self,
        ctx: &Context,
        action: &AppAction,
        action_tx: UnboundedSender<AppAction>,
    ) -> Result<()> {
        if self.loading_state.is_playing() && self.results.is_none() {
            self.loading_state.update(ctx.render_delta_time);
        } else {
            self.loading_state.goto_start();
        }
        if self.loading_state.is_done() {
            self.loading_state.goto_start();
        }

        match action {
            AppAction::Task(TaskAction::SourceResults(results)) => match results {
                Ok(results) => {
                    self.results.clone_from(results);
                    self.table_state.select_first();
                }
                Err(msg) => {
                    action_tx.send(AppAction::Error(msg.to_string()))?;
                }
            },
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

        let mult = ctx
            .keycombo
            .repeat()
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_default();
        let keycombo = ctx
            .keycombo
            .events()
            .iter()
            .map(keys::key_event_to_string)
            .collect::<String>();
        let keycombo_color = ctx.keycombo.status().color();

        self.current_keycombo = format!("{mult}{keycombo}");
        self.current_keycombo_color = keycombo_color;

        Ok(())
    }

    fn render(&mut self, ctx: &Context, frame: &mut Frame, area: Rect) -> Result<()> {
        let mut block = Block::new()
            .fg(Color::Rgb(255, 255, 255))
            .borders(Borders::ALL);
        let vr = line::NORMAL.vertical_right;
        let vl = line::NORMAL.vertical_left;
        if !self.current_keycombo.is_empty() {
            let combo = self
                .current_keycombo
                .as_str()
                .fg(self.current_keycombo_color);
            let keycombo = Line::from_iter([
                format!("{vl} ").fg(Color::Rgb(255, 255, 255)),
                combo,
                format!(" {vr}").fg(Color::Rgb(255, 255, 255)),
            ]);
            block = block.title_bottom(keycombo.right_aligned())
        };

        let mode = Line::from_iter([
            format!("{vl} ").fg(Color::Rgb(255, 255, 255)),
            ctx.input_mode.to_string().fg(Color::Cyan.to_rgb()),
            ".".to_string().fg(Color::White.to_rgb()),
            ctx.mode.to_string().fg(Color::Cyan.to_rgb()),
            format!(" {vr}").fg(Color::Rgb(255, 255, 255)),
        ]);
        block = block.title_bottom(mode);

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
            let loading_frame = (self.loading_state.get_smooth_time()
                * (LOADING_CHARS.len() as f64))
                .floor() as usize;
            let loading_char = LOADING_CHARS[loading_frame.min(LOADING_CHARS.len() - 1)];
            let text = format!("{loading_char} Loading...");
            let paragraph = Paragraph::new(text.as_str());
            let center = super::centered_rect(area, text.len() as u16, 1);
            frame.render_widget(paragraph, center);
        }

        Ok(())
    }
}
