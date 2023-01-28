use crate::app::{App, InputMode};
use crate::nyaa;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, BorderType, Borders, Cell, Clear, Paragraph, Row, Table, Wrap},
    Frame,
};
use unicode_width::UnicodeWidthStr;
use queues::IsQueue;

pub fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let size = f.size();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(f.size());

    let def_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default())
        .border_type(BorderType::Rounded);

    let hi_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::LightCyan))
        .border_type(BorderType::Rounded);

    let help_message = create_message(app);
    f.render_widget(help_message, chunks[0]);

    let input = create_search_bar(app, &hi_block, &def_block);
    f.render_widget(input, chunks[1]);

    let table = create_table(app, &hi_block, &def_block);
    f.render_stateful_widget(table, chunks[2], &mut app.items.state.to_owned());
    
    match app.input_mode {
        InputMode::Editing => {
            f.set_cursor(chunks[1].x + app.input.width() as u16 + 1, chunks[1].y + 1)
        }
        InputMode::SelectCategory => {
            let popup = create_popup(
                app,
                app.categories.items.to_owned(),
                app.category as usize,
                &hi_block,
                &def_block,
                InputMode::SelectCategory,
                "Categories".to_owned(),
            );
            let area = centered_rect(30, 7, size);
            f.render_widget(Clear, area);
            f.render_stateful_widget(popup, area, &mut app.categories.state);
        }
        InputMode::SelectFilter => {
            let popup = create_popup(
                app,
                app.filters.items.to_owned(),
                app.filter as usize,
                &hi_block,
                &def_block,
                InputMode::SelectFilter,
                "Filters".to_owned(),
            );
            let area = centered_rect(30, 7, size);
            f.render_widget(Clear, area);
            f.render_stateful_widget(popup, area, &mut app.filters.state);
        }
        InputMode::SelectSort => {
            let popup = create_popup(
                app,
                app.sorts.items.to_owned(),
                app.sort as usize,
                &hi_block,
                &def_block,
                InputMode::SelectFilter,
                "Sort".to_owned(),
            );
            let area = centered_rect(30, 8, size);
            f.render_widget(Clear, area);
            f.render_stateful_widget(popup, area, &mut app.sorts.state);
        }
        _ => {}
    }

    // Check for any errors
    if !app.errors.size() != 0 {
        if let Ok(err) = app.errors.peek() {
            app.last_input_mode = app.input_mode.to_owned();
            app.input_mode = InputMode::ShowError;
            
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red))
                .border_type(BorderType::Rounded);
            
            let popup = create_text_popup(err + "\n\nPress any key to dismiss...", "Error".to_owned(), &block);
            let area = centered_rect(60, 20, size);
            f.render_widget(Clear, area);
            f.render_widget(popup, area);
        }
    }
}

fn create_message(app: &App) -> Paragraph {
    let (msg, style) = match app.input_mode {
        InputMode::Normal => (
            vec![
                Span::raw("Press "),
                Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to exit, "),
                Span::styled("/", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to search, "),
                Span::styled("hjkl", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" for movement, "),
                Span::styled("c", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" for categories, "),
                Span::styled("f", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" for filters"),
            ],
            Style::default(),
        ),
        InputMode::Editing => (
            vec![
                Span::raw("Press "),
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to stop typing, "),
                Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to search"),
            ],
            Style::default(),
        ),
        InputMode::SelectFilter | InputMode::SelectCategory | InputMode::SelectSort => (
            vec![
                Span::raw("Press "),
                Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to exit, "),
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to leave popup, "),
                Span::styled("hjkl", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" for movement"),
                Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to confirm selection"),
            ],
            Style::default(),
        ),
        InputMode::ShowError => (
            vec![
                Span::raw("Press "),
                Span::styled("any key", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to dismiss"),
            ],
            Style::default(),
        )
    };
    let mut text = Text::from(Spans::from(msg));
    text.patch_style(style);
    Paragraph::new(text)
}

fn create_search_bar<'a>(app: &'a App, hi_block: &'a Block, def_block: &'a Block) -> Paragraph<'a> {
    Paragraph::new(app.input.as_ref()).block(
        match app.input_mode {
            InputMode::Editing => hi_block.clone(),
            _ => def_block.clone(),
        }
        .title("Search"),
    )
}

fn create_table<'a>(app: &'a App, hi_block: &'a Block<'a>, def_block: &'a Block<'a>) -> Table<'a> {
    let header_cells = ["Cat", "Name", "", "", ""]
        .iter()
        .map(|h| Cell::from(Text::raw(*h)).style(Style::default().add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells)
        .style(
            Style::default()
                .add_modifier(Modifier::UNDERLINED)
                .fg(Color::White),
        )
        .height(1)
        .bottom_margin(0);

    let items = app.items.items.iter().map(|item| {
        Row::new(vec![
            item.category.get_icon(),
            item.get_styled_title(),
            Text::styled(shorten_number(item.seeders), Style::default().fg(Color::Green)),
            Text::styled(shorten_number(item.leechers), Style::default().fg(Color::Red)),
            Text::from(shorten_number(item.downloads)),
        ])
        .height(1)
        .bottom_margin(0)
    });

    Table::new(items)
        .header(header)
        .block(match app.input_mode {
            InputMode::Normal => hi_block.to_owned(),
            _ => def_block.to_owned(),
        })
        .highlight_style(
            Style::default()
                .bg(Color::White)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .widths(&[
            Constraint::Percentage(3),
            Constraint::Percentage(84),
            Constraint::Percentage(4),
            Constraint::Percentage(4),
            Constraint::Percentage(5),
        ])
}

fn create_text_popup<'a>(
    text: String,
    title: String,
    block: &'a Block,
) -> Paragraph<'a> {
    
    Paragraph::new(text)
        .block(block.to_owned().title(title))
        .wrap(Wrap { trim: false })
}

fn create_popup<'a, T: nyaa::Named + num::FromPrimitive + Default + PartialEq>(
    app: &App,
    items: Vec<T>,
    sel_idx: usize,
    hi_block: &'a Block,
    def_block: &'a Block,
    mode: InputMode,
    title: String,
) -> Table<'a> {
    let n: T = num::FromPrimitive::from_usize(sel_idx).unwrap_or_default();
    let items = items.iter().map(|item| {
        let sel = if &n == item { "[x] " } else { "[ ] " }.to_owned();
        Row::new(vec![sel + &item.get_name()])
    });
    Table::new(items)
        .block(
            match &app.input_mode {
                _ if app.input_mode == mode => hi_block.to_owned(),
                _ => def_block.to_owned(),
            }
            .title(title),
        )
        .highlight_style(
            Style::default()
                .bg(Color::White)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .widths(&[Constraint::Percentage(100)])
}

fn shorten_number(mut n: u32) -> String {
    if n >= 10000 {
        n /= 1000;
        return n.to_string() + "K";
    }
    n.to_string()
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                // Constraint::Percentage((100 - percent_y) / 2),
                // Constraint::Percentage(percent_y),
                // Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Length((r.height - percent_y) / 2),
                Constraint::Length(percent_y),
                Constraint::Length((r.height - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                // Constraint::Percentage((100 - percent_x) / 2),
                // Constraint::Percentage(percent_x),
                // Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Length((r.width - percent_x) / 2),
                Constraint::Length(percent_x),
                Constraint::Length((r.width - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
