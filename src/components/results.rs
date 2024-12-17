use color_eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Alignment, Rect},
    widgets::{Block, Borders, StatefulWidget, Table, TableState},
    Frame,
};

use crate::{
    action::AppAction,
    app::{Context, Mode},
    result::{ResultHeader, ResultTable},
};

use super::Component;

pub struct ResultsComponent {
    results: ResultTable,
}

impl ResultsComponent {
    pub fn new() -> Self {
        let header: &mut [ResultHeader] = &mut [
            ("Thank", 'x').into(),
            ("you", 'x').into(),
            ("for", 'x').into(),
            ("watching", 'x').into(),
            ("me", 'x').into(),
        ];
        header
            .iter_mut()
            .for_each(|h| h.set_alignment(Alignment::Center));
        Self {
            results: ResultTable::new([
                ["This", "is", "a", "test"],
                ["This", "is", "a", "test"],
                ["This", "is", "a", "test"],
                ["This", "is", "a", "test"],
                ["This", "is", "a", "test"],
                ["This", "is", "a", "test"],
            ])
            .header(header.to_vec())
            .apply_alignment([
                Alignment::Left,
                Alignment::Center,
                Alignment::Right,
                Alignment::Left,
            ]),
        }
    }
}

impl Component for ResultsComponent {
    fn update(
        &mut self,
        _ctx: &Context,
        _action: &AppAction,
    ) -> color_eyre::eyre::Result<Option<AppAction>> {
        Ok(None)
    }

    fn on_key(&mut self, ctx: &Context, _key: &KeyEvent) -> Result<()> {
        if ctx.mode != Mode::Home {
            return Ok(());
        }
        Ok(())
    }

    fn render(&mut self, _ctx: &Context, frame: &mut Frame, area: Rect) -> Result<()> {
        let block = Block::new().borders(Borders::ALL);
        let table: Table = self.results.clone().into();
        table
            .block(block)
            .render(area, frame.buffer_mut(), &mut TableState::default());
        Ok(())
    }
}
