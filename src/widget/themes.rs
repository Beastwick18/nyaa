use std::cmp::min;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Margin, Rect},
    widgets::{Row, ScrollbarOrientation, StatefulWidget as _, Table},
    Frame,
};

use crate::{
    app::{Context, Mode},
    style, title,
};

use super::{border_block, VirtualStatefulTable, Widget};

pub struct ThemePopup {
    pub table: VirtualStatefulTable,
    pub selected: usize,
}

impl Default for ThemePopup {
    fn default() -> Self {
        ThemePopup {
            table: VirtualStatefulTable::new(),
            selected: 0,
        }
    }
}

fn preview_theme(idx: usize, ctx: &mut Context) {
    if let Some((_, theme)) = ctx.themes.get_index(idx) {
        ctx.theme = theme.clone();
        ctx.results.table = ctx.src.format_table(
            &ctx.results.response.items,
            &ctx.results.search,
            &ctx.config.sources,
            &ctx.theme,
        );
    }
}

impl Widget for ThemePopup {
    fn draw(&mut self, f: &mut Frame, ctx: &Context, area: Rect) {
        let buf = f.buffer_mut();
        let height = min(min(ctx.themes.len() as u16 + 2, 10), area.height);
        let center = super::centered_rect(30, height, area);
        let items = ctx.themes.keys().enumerate().map(|(i, item)| {
            Row::new(vec![
                match i == self.selected {
                    true => format!("  {}", item),
                    false => format!("   {}", item),
                },
                item.to_owned(),
            ])
        });

        let num_items = items.len();
        super::scroll_padding(
            self.table.selected().unwrap_or(0),
            center.height as usize,
            2,
            num_items,
            1,
            self.table.state.offset_mut(),
        );

        let table = Table::new(items, [Constraint::Percentage(100)])
            .block(border_block(&ctx.theme, true).title(title!("Theme")))
            .highlight_style(style!(bg:ctx.theme.hl_bg));
        super::clear(center, buf, ctx.theme.bg);
        table.render(center, buf, &mut self.table.state);

        // Only show scrollbar if content overflows
        if ctx.themes.len() as u16 + 1 >= center.height {
            let sb = super::scrollbar(ctx, ScrollbarOrientation::VerticalRight);
            let sb_area = center.inner(Margin {
                vertical: 1,
                horizontal: 0,
            });
            sb.render(
                sb_area,
                buf,
                &mut self.table.scrollbar_state.content_length(num_items),
            );
        }
    }

    fn handle_event(&mut self, ctx: &mut Context, e: &Event) {
        if let Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            ..
        }) = e
        {
            match code {
                KeyCode::Esc | KeyCode::Char('t') | KeyCode::Char('q') => {
                    ctx.mode = Mode::Normal;
                    if Some(self.selected) != self.table.selected() {
                        preview_theme(self.selected, ctx);
                    }
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    let idx = self.table.next_wrap(ctx.themes.len(), 1);
                    preview_theme(idx, ctx);
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    let idx = self.table.next_wrap(ctx.themes.len(), -1);
                    preview_theme(idx, ctx);
                }
                KeyCode::Char('G') => {
                    let idx = ctx.themes.len().saturating_sub(1);
                    self.table.select(idx);
                    preview_theme(idx, ctx);
                }
                KeyCode::Char('g') => {
                    self.table.select(0);
                    preview_theme(0, ctx);
                }
                KeyCode::Enter => {
                    let idx = self.table.selected().unwrap_or(0);
                    if let Some((_, theme)) = ctx.themes.get_index(idx) {
                        let theme_name = theme.name.clone();
                        self.selected = idx;
                        ctx.theme = theme.clone();
                        ctx.config.theme.clone_from(&theme.name);
                        ctx.results.table = ctx.src.format_table(
                            &ctx.results.response.items,
                            &ctx.results.search,
                            &ctx.config.sources,
                            &ctx.theme,
                        );
                        match ctx.save_config() {
                            Ok(_) => {
                                ctx.notify_info(format!("Updated theme to \"{}\"", theme_name))
                            }
                            Err(e) => ctx.notify_error(format!(
                                "Failed to update default theme in config file:\n{}",
                                e
                            )),
                        }
                    }
                    ctx.mode = Mode::Normal;
                }
                _ => {}
            }
        }
    }

    fn get_help() -> Option<Vec<(&'static str, &'static str)>> {
        Some(vec![
            ("Enter", "Confirm"),
            ("Esc, t, q", "Close"),
            ("j, ↓", "Down"),
            ("k, ↑", "Up"),
            ("g", "Top"),
            ("G", "Bottom"),
        ])
    }
}
