use crate::nyaa;
use tui::widgets::{TableState, Paragraph};
use queues::Queue;
use crossterm::event::KeyCode;
use crate::nyaa::EnumIter;

pub struct Popup<T: Default> {
    pub table: StatefulTable<T>,
    pub selected: T
}

impl<T: Default + Clone> Popup<T> {
    fn with_items(items: Vec<T>) -> Popup<T> {
        Popup {
            table: StatefulTable::with_items(items),
            selected: T::default()
        }
    }
    
    pub fn handle_keybinds<F>(&mut self, last_input_mode: InputMode, key: KeyCode, on_confirm: F) -> Option<InputMode> where F: FnOnce(usize, &T) {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => {
                return Some(last_input_mode.to_owned())
            }
            KeyCode::Char('/') => return Some(InputMode::Editing),
            KeyCode::Char('j') | KeyCode::Down => self.table.next(1),
            KeyCode::Char('k') | KeyCode::Up => self.table.previous(1),
            KeyCode::Char('J') => self.table.next(4),
            KeyCode::Char('K') => self.table.previous(4),
            KeyCode::Char('g') => self.table.select(0),
            KeyCode::Char('G') => self.table.select(self.table.items.len() - 1),
            KeyCode::Enter | KeyCode::Char('l') => {
                if let Some(i) = self.table.state.selected() {
                    if let Some(item) = self.table.items.get(i).clone() {
                        self.selected = item.clone();
                        on_confirm(i, &item);
                        // app.category = item.to_owned().to_owned();
                        // app.input_mode = app.last_input_mode.to_owned();
                        // search_nyaa(&mut app).await;
                    }
                }
            }
            _ => {}
        };
        None
    }
}

pub struct StatefulTable<T> {
    pub state: TableState,
    pub items: Vec<T>,
}

impl<T> StatefulTable<T> {
    pub fn with_items(items: Vec<T>) -> StatefulTable<T> {
        StatefulTable {
            state: TableState::default(),
            items,
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
                if i < amt {
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
    pub last_input_mode: InputMode,
    /// History of recorded messages
    pub items: StatefulTable<nyaa::Item>,
    pub category: Popup<nyaa::Category>,
    pub filter: Popup<nyaa::Filter>,
    pub sort: Popup<nyaa::Sort>,
    pub errors: Queue<String>,
    pub help: String
}

#[derive(Clone, PartialEq)]
pub enum InputMode {
    Normal,
    Editing,
    ShowError,
    SelectCategory,
    SelectFilter,
    SelectSort,
    ShowHelp,
}

impl Default for App {
    fn default() -> App {
        App {
            input: String::new(),
            input_mode: InputMode::Editing,
            last_input_mode: InputMode::Editing,
            items: StatefulTable::with_items(Vec::new()),
            category: Popup::<nyaa::Category>::with_items(
                nyaa::Category::iter().map(|item| item.to_owned()).collect(),
            ),
            filter: Popup::<nyaa::Filter>::with_items(
                nyaa::Filter::iter().map(|item| item.to_owned()).collect()
            ),
            sort: Popup::<nyaa::Sort>::with_items(
                nyaa::Sort::iter().map(|item| item.to_owned()).collect()
            ),
            // category: nyaa::Category::default(),
            // filter: nyaa::Filter::default(),
            // sort: nyaa::Sort::default(),
            // categories: StatefulTable::with_items(
            //     nyaa::Category::iter().map(|item| item.to_owned()).collect(),
            // ),
            // filters: StatefulTable::with_items(
            //     nyaa::Filter::iter().map(|item| item.to_owned()).collect(),
            // ),
            // sorts: StatefulTable::with_items(
            //     nyaa::Sort::iter().map(|item| item.to_owned()).collect()
            // ),
            errors: Queue::default(),
            help:  
"Normal mode:      | Editing mode:
q = Quit          | Esc = Stop editing 
/ = Search        | Enter = Submit
hjkl/ = move  +--------------------
c = Pick category | Popup mode:
f = Pick filter   | q/Esc = Close
s = Sort select   | hjlk/ = move
                  | Enter = Confirm".to_owned()
        }
    }
}
