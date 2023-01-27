use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, BorderType, Cell, Table, Row, TableState, Paragraph},
    Frame
};
use crate::nyaa;
use unicode_width::UnicodeWidthStr;

pub struct StatefulTable<T> {
    pub state: TableState,
    pub items: Vec<T>,
}

impl<T> StatefulTable<T> {
    pub fn with_items(items: Vec<T>) -> StatefulTable<T> {
        StatefulTable {
            state: TableState::default(),
            items
        }
    }

    pub fn next(&mut self, amt: usize) {
        if self.items.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i + amt >= self.items.len() {
                    self.items.len() - 1
                } else {
                    i + amt
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self, amt: usize) {
        if self.items.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i+1 <= amt  {
                    0
                } else {
                    i - amt
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
    
    pub fn select(&mut self, idx: usize) {
        self.state.select(Some(idx));
    }

    // fn unselect(&mut self) {
    //     self.state.select(None);
    // }
}

pub struct App {
    /// Current value of the input box
    // #[derive(Clone)]
    pub input: String,
    /// Current input mode
    pub input_mode: InputMode,
    /// History of recorded messages
    pub items: StatefulTable<nyaa::Item>,
    // pub handle: Option<JoinHandle<Vec<nyaa::Item>>>,
    pub category: nyaa::Category,
    pub filter: nyaa::Filter
}

pub enum InputMode {
    Normal,
    Editing,
}

impl Default for App {
    fn default() -> App {
        App {
            input: String::new(),
            input_mode: InputMode::Editing,
            items: StatefulTable::with_items(Vec::new()),
            category: nyaa::Category::AllAnime,
            filter: nyaa::Filter::NoFilter,
        }
    }
}

pub fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
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
    let def_block = Block::default().borders(Borders::ALL)
        .border_style(Style::default())
        .border_type(BorderType::Rounded);
    
    let hi_block = Block::default().borders(Borders::ALL)
        .border_style(Style::default().fg(Color::LightCyan))
        .border_type(BorderType::Rounded);

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
    };
    let mut text = Text::from(Spans::from(msg));
    text.patch_style(style);
    let help_message = Paragraph::new(text);
    f.render_widget(help_message, chunks[0]);

    let input = Paragraph::new(app.input.as_ref())
        .block(match app.input_mode {
            InputMode::Normal => def_block.clone(),
            InputMode::Editing => hi_block.clone()
        }.title("Search"));
    f.render_widget(input, chunks[1]);
    match app.input_mode {
        InputMode::Normal =>
            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
            {}

        InputMode::Editing => {
            // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
            f.set_cursor(
                // Put cursor past the end of the input text
                chunks[1].x + app.input.width() as u16 + 1,
                // Move one line down, from the border to the input line
                chunks[1].y + 1,
            )
        }
    }

    let header_cells = ["Cat", "Name", "", "", ""]
        .iter()
        .map(|h| Cell::from(Text::raw(*h)).style(Style::default().add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells)
        .style(Style::default().add_modifier(Modifier::UNDERLINED).fg(Color::White))
        .height(1)
        .bottom_margin(0);
    
    let items = app
        .items
        .items
        .iter()
        .map(|item| {
            let cells = vec![item.category.get_icon(), item.get_styled_title(), Text::from(shorten_number(item.seeders)), Text::from(shorten_number(item.leechers)), Text::from(shorten_number(item.downloads))];
            Row::new(cells).height(1).bottom_margin(0)
        });
    
    let items = Table::new(items)
        .header(header)
        .block(match app.input_mode {
            InputMode::Normal => hi_block.clone(),
            InputMode::Editing => def_block.clone()
        })
        .highlight_style(
            Style::default()
                .bg(Color::White)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        ).widths(&[
            Constraint::Percentage(3),
            Constraint::Percentage(87),
            Constraint::Percentage(3),
            Constraint::Percentage(3),
            Constraint::Percentage(4)
        ]);
    
    
    f.render_stateful_widget(items, chunks[2], &mut app.items.state);
}

fn shorten_number(mut n: u32) -> String {
    if n >= 10000 {
        n /= 1000;
        return n.to_string() + "K";
    }
    n.to_string()
}

// fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
//     let popup_layout = Layout::default()
//         .direction(Direction::Vertical)
//         .constraints(
//             [
//                 Constraint::Percentage((100 - percent_y) / 2),
//                 Constraint::Percentage(percent_y),
//                 Constraint::Percentage((100 - percent_y) / 2),
//             ]
//             .as_ref(),
//         )
//         .split(r);

//     Layout::default()
//         .direction(Direction::Horizontal)
//         .constraints(
//             [
//                 Constraint::Percentage((100 - percent_x) / 2),
//                 Constraint::Percentage(percent_x),
//                 Constraint::Percentage((100 - percent_x) / 2),
//             ]
//             .as_ref(),
//         )
//         .split(popup_layout[1])[1]
// }
