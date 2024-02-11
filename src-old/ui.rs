use crate::app::{App, InputMode};
use queues::IsQueue;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Cell, Clear, Paragraph, Row, Table, Wrap},
    Frame,
};
use unicode_width::UnicodeWidthStr;

static BORDER: BorderType = BorderType::Plain;

#[macro_export]
macro_rules! bold {
    ( $x:expr ) => {
        Span::styled($x, Style::default().add_modifier(Modifier::BOLD))
    };
}

#[macro_export]
macro_rules! raw {
    ( $x:expr ) => {
        Span::raw($x)
    };
}

pub fn ui<B: Backend>(f: &mut Frame, app: &mut App) {
    let size = f.size();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints(&[
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .split(f.size());

    let def_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default())
        .border_type(BORDER);

    let hi_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::LightCyan))
        .border_type(BORDER);

    let empty_block = Block::default().borders(Borders::NONE);

    let help_message = create_message(app);
    f.render_widget(help_message, chunks[0]);

    let input = create_search_bar(app, &hi_block, &def_block);
    f.render_widget(input, chunks[1]);

    let binding = [
        Constraint::Length(3),
        Constraint::Length(chunks[2].width - 21),
        Constraint::Length(4),
        Constraint::Length(4),
        Constraint::Length(5),
    ];
    let table = create_table(app, &hi_block, &def_block).widths(&binding);
    f.render_stateful_widget(table, chunks[2], &mut app.table.state.to_owned());

    match app.input_mode {
        InputMode::Editing => {
            f.set_cursor(chunks[1].x + app.input.width() as u16 + 1, chunks[1].y + 1)
        }
        InputMode::SelectCategory => {
            let popup = create_popup(
                app,
                app.category.table.items.to_owned(),
                app.category.selected as usize,
                &hi_block,
                &def_block,
                InputMode::SelectCategory,
                "Categories".to_owned(),
            );

            let area = centered_rect(30, app.category.table.items.len() as u16 + 2, size);
            f.render_widget(Clear, area);
            f.render_stateful_widget(popup, area, &mut app.category.table.state);
        }
        InputMode::SelectFilter => {
            let popup = create_popup(
                app,
                app.filter.table.items.to_owned(),
                app.filter.selected as usize,
                &hi_block,
                &def_block,
                InputMode::SelectFilter,
                "Filters".to_owned(),
            );
            let area = centered_rect(30, app.filter.table.items.len() as u16 + 2, size);
            f.render_widget(Clear, area);
            f.render_stateful_widget(popup, area, &mut app.filter.table.state);
        }
        InputMode::SelectSort => {
            let popup = create_popup(
                app,
                app.sort.table.items.to_owned(),
                app.sort.selected as usize,
                &hi_block,
                &def_block,
                InputMode::SelectSort,
                "Sort".to_owned(),
            );
            let area = centered_rect(30, app.sort.table.items.len() as u16 + 2, size);
            f.render_widget(Clear, area);
            f.render_stateful_widget(popup, area, &mut app.sort.table.state);
        }
        InputMode::ShowHelp => {
            let popup = create_text_popup(app.help.to_owned(), "Help".to_owned(), &hi_block);
            let area = centered_rect(41, 10, size);
            f.render_widget(Clear, area);
            f.render_widget(popup, area);
        }
        InputMode::Loading => {
            app.input_mode = InputMode::Searching;
            let popup = create_text_popup("".to_owned(), "Loading...".to_owned(), &empty_block);
            let area = centered_rect(10, 1, size);
            f.render_widget(Clear, area);
            f.render_widget(popup, area);
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
                .border_type(BORDER);

            let popup = create_text_popup(
                err + "\n\nPress any key to dismiss...",
                "Error".to_owned(),
                &block,
            );
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
                raw!("Press "),
                bold!("q"),
                raw!(" to exit, "),
                bold!("/"),
                raw!(" to search, "),
                bold!("F1"),
                raw!(" for keybinds, "),
                bold!("Enter"),
                raw!(" to download"),
            ],
            Style::default(),
        ),
        InputMode::Editing => (
            vec![
                raw!("Press "),
                bold!("Esc"),
                raw!(" to stop typing, "),
                bold!("Enter"),
                raw!(" to search"),
            ],
            Style::default(),
        ),
        InputMode::SelectFilter | InputMode::SelectCategory | InputMode::SelectSort => (
            vec![
                raw!("Press "),
                bold!("q"),
                raw!(" to exit, "),
                bold!("Esc"),
                raw!(" to leave popup, "),
                bold!("hjkl"),
                raw!(" for movement, "),
                bold!("Enter"),
                raw!(" to confirm selection"),
            ],
            Style::default(),
        ),
        InputMode::ShowError | InputMode::ShowHelp => (
            vec![raw!("Press "), bold!("any key"), raw!(" to dismiss")],
            Style::default(),
        ),
        InputMode::Loading | InputMode::Searching => (vec![], Style::default()),
    };
    let mut text = Line::from(msg);
    text.patch_style(style);
    Paragraph::new(text)
}

fn create_search_bar<'a>(app: &'a App, hi_block: &'a Block, def_block: &'a Block) -> Paragraph<'a> {
    Paragraph::new(app.input.as_str()).block(
        match app.input_mode {
            InputMode::Editing => hi_block.clone(),
            _ => def_block.clone(),
        }
        .title("Search"),
    )
}

fn create_table<'a>(app: &'a App, hi_block: &'a Block<'a>, def_block: &'a Block<'a>) -> Table<'a> {
    static HEADER_CELLS: [&str; 5] = ["Cat", "Name", "", "", "󰇚"];
    let header_cells = HEADER_CELLS
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

    let items = app.table.items.iter().map(|item| {
        Row::new(vec![
            item.category.get_icon(),
            item.get_styled_title(),
            Text::styled(
                shorten_number(item.seeders),
                Style::default().fg(Color::Green),
            ),
            Text::styled(
                shorten_number(item.leechers),
                Style::default().fg(Color::Red),
            ),
            Text::from(shorten_number(item.downloads)),
        ])
        .height(1)
        .bottom_margin(0)
    });

    Table::new(items, [Constraint::Percentage(100)])
        .header(header)
        .block(match app.input_mode {
            InputMode::Normal => hi_block.to_owned(),
            _ => def_block.to_owned(),
        })
        .highlight_style(Style::default().bg(Color::Rgb(60, 60, 60)))
}

fn create_text_popup<'a>(text: String, title: String, block: &'a Block) -> Paragraph<'a> {
    Paragraph::new(text)
        .block(block.to_owned().title(title))
        .wrap(Wrap { trim: false })
}

fn create_popup<'a, T: ToString + num::FromPrimitive + Default + PartialEq>(
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
        Row::new(vec![sel + &item.to_string()])
    });
    Table::new(items, &[Constraint::Percentage(100)])
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
}

fn shorten_number(mut n: u32) -> String {
    if n >= 10000 {
        n /= 1000;
        return n.to_string() + "K";
    }
    n.to_string()
}

fn centered_rect(x_len: u16, y_len: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(&[
            Constraint::Length((r.height - y_len) / 2),
            Constraint::Length(y_len),
            Constraint::Length((r.height - y_len) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(&[
            Constraint::Length((r.width - x_len) / 2),
            Constraint::Length(x_len),
            Constraint::Length((r.width - x_len) / 2),
        ])
        .split(popup_layout[1])[1]
}
