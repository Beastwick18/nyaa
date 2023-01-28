use crate::nyaa;
use tui::widgets::TableState;
use queues::Queue;

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
    // pub handle: Option<JoinHandle<Vec<nyaa::Item>>>,
    pub category: nyaa::Category,
    pub filter: nyaa::Filter,
    pub sort: nyaa::Sort,
    pub categories: StatefulTable<nyaa::Category>,
    pub filters: StatefulTable<nyaa::Filter>,
    pub sorts: StatefulTable<nyaa::Sort>,
    pub errors: Queue<String>
}

#[derive(Clone, PartialEq)]
pub enum InputMode {
    Normal,
    Editing,
    ShowError,
    SelectCategory,
    SelectFilter,
    SelectSort,
}

impl Default for App {
    fn default() -> App {
        App {
            input: String::new(),
            input_mode: InputMode::Editing,
            last_input_mode: InputMode::Editing,
            items: StatefulTable::with_items(Vec::new()),
            category: nyaa::Category::default(),
            filter: nyaa::Filter::default(),
            sort: nyaa::Sort::default(),
            categories: StatefulTable::with_items(
                nyaa::Category::iter().map(|item| item.to_owned()).collect(),
            ),
            filters: StatefulTable::with_items(
                nyaa::Filter::iter().map(|item| item.to_owned()).collect(),
            ),
            sorts: StatefulTable::with_items(
                nyaa::Sort::iter().map(|item| item.to_owned()).collect()
            ),
            errors: Queue::default()
        }
    }
}
