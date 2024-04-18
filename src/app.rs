use std::{collections::VecDeque, error::Error};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use indexmap::IndexMap;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    Frame, Terminal,
};

use crate::{
    client::Client,
    clip,
    config::Config,
    source::{Item, Sources},
    theme::{self, Theme},
    util::key_to_string,
    widget::{
        batch::BatchWidget,
        category::CategoryPopup,
        clients::ClientsPopup,
        error::ErrorPopup,
        filter::FilterPopup,
        help::HelpPopup,
        page::PagePopup,
        results::ResultsWidget,
        search::SearchWidget,
        sort::{SortDir, SortPopup},
        sources::SourcesPopup,
        themes::ThemePopup,
        user::UserPopup,
        Widget,
    },
};

#[cfg(unix)]
use core::panic;
#[cfg(unix)]
use crossterm::event::KeyModifiers;

#[cfg(unix)]
use crate::util::{continue_self, suspend_self};

pub static APP_NAME: &str = "nyaa";

#[derive(PartialEq, Clone, Copy)]
pub enum LoadType {
    Searching,
    Sorting,
    Filtering,
    Categorizing,
    Batching,
    Downloading,
}

#[derive(PartialEq, Clone)]
pub enum Mode {
    Normal,
    KeyCombo(Vec<char>),
    Search,
    Category,
    Sort(SortDir),
    Batch,
    Filter,
    Theme,
    Sources,
    Clients,
    Loading(LoadType),
    Error,
    Page,
    User,
    Help,
}

impl ToString for Mode {
    fn to_string(&self) -> String {
        match self {
            Mode::Normal | Mode::KeyCombo(_) => "Normal".to_owned(),
            Mode::Batch => "Batch".to_owned(),
            Mode::Search => "Search".to_owned(),
            Mode::Category => "Category".to_owned(),
            Mode::Sort(_) => "Sort".to_owned(),
            Mode::Filter => "Filter".to_owned(),
            Mode::Theme => "Theme".to_owned(),
            Mode::Sources => "Sources".to_owned(),
            Mode::Clients => "Clients".to_owned(),
            Mode::Loading(_) => "Loading".to_owned(),
            Mode::Error => "Error".to_owned(),
            Mode::Page => "Page".to_owned(),
            Mode::User => "User".to_owned(),
            Mode::Help => "Help".to_owned(),
        }
    }
}

#[derive(Default)]
pub struct App {
    pub widgets: Widgets,
}

pub struct Context {
    pub mode: Mode,
    pub themes: IndexMap<String, Theme>,
    pub theme: Theme,
    pub config: Config,
    pub errors: VecDeque<String>,
    pub notification: Option<String>,
    pub ascending: bool,
    pub page: usize,
    pub user: Option<String>,
    pub last_page: usize,
    pub total_results: usize,
    pub src: Sources,
    pub client: Client,
    pub batch: Vec<Item>,
    pub last_key: String,
    should_quit: bool,
}

impl Context {
    pub fn show_error<S: ToString>(&mut self, error: S) {
        self.errors.push_back(error.to_string());
    }

    pub fn notify<S: ToString>(&mut self, notification: S) {
        self.notification = Some(notification.to_string());
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}

impl Default for Context {
    fn default() -> Self {
        let themes = theme::default_themes();
        Context {
            mode: Mode::Loading(LoadType::Searching),
            themes,
            theme: Theme::default(),
            config: Config::default(),
            errors: VecDeque::new(),
            notification: None,
            ascending: false,
            page: 1,
            user: None,
            last_page: 1,
            total_results: 0,
            src: Sources::NyaaHtml,
            client: Client::Cmd,
            batch: vec![],
            last_key: "".to_owned(),
            should_quit: false,
        }
    }
}

#[derive(Default)]
pub struct Widgets {
    pub batch: BatchWidget,
    pub category: CategoryPopup,
    pub sort: SortPopup,
    pub filter: FilterPopup,
    pub theme: ThemePopup,
    pub sources: SourcesPopup,
    pub clients: ClientsPopup,
    pub search: SearchWidget,
    pub results: ResultsWidget,
    pub error: ErrorPopup,
    pub page: PagePopup,
    pub user: UserPopup,
    pub help: HelpPopup,
}

impl App {
    pub async fn run_app<B: Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
    ) -> Result<(), Box<dyn Error>> {
        let w = &mut Widgets::default();
        let ctx = &mut Context::default();
        let config = match Config::load() {
            Ok(config) => config,
            Err(e) => {
                ctx.show_error(e);
                ctx.config.clone()
            }
        };
        config.apply(ctx, w);
        while !ctx.should_quit {
            if !ctx.errors.is_empty() {
                ctx.mode = Mode::Error;
            }
            if ctx.mode == Mode::Batch && ctx.batch.is_empty() {
                ctx.mode = Mode::Normal;
            }

            self.get_help(w, ctx);
            terminal.draw(|f| self.draw(w, ctx, f))?;
            if let Mode::Loading(load_type) = ctx.mode {
                ctx.mode = Mode::Normal;
                match load_type {
                    LoadType::Downloading => {
                        if load_type == LoadType::Downloading {
                            if let Some(i) = w.results.table.selected() {
                                ctx.client.clone().download(i.to_owned(), ctx).await;
                            }
                            continue;
                        }
                    }
                    LoadType::Batching => {
                        ctx.client
                            .clone()
                            .batch_download(ctx.batch.clone(), ctx)
                            .await;
                        continue;
                    }
                    _ => {}
                }

                let result = ctx.src.clone().load(load_type, ctx, w).await;

                match result {
                    Ok(items) => w.results.with_items(items, w.sort.selected),
                    Err(e) => ctx.show_error(e),
                }
                continue; // Redraw
            }

            let evt = event::read()?;
            #[cfg(unix)]
            self.on(&evt, w, ctx, terminal);
            #[cfg(not(unix))]
            self.on::<B>(&evt, w, ctx);
        }
        Ok(())
    }

    pub fn draw(&mut self, widgets: &mut Widgets, ctx: &mut Context, f: &mut Frame) {
        let layout_vertical = Layout::new(
            Direction::Vertical,
            [Constraint::Length(3), Constraint::Min(1)],
        )
        .split(f.size());
        let layout_horizontal = Layout::new(
            Direction::Horizontal,
            match ctx.mode {
                Mode::Batch => [Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)],
                _ => [Constraint::Ratio(3, 4), Constraint::Ratio(1, 4)],
            },
        )
        .split(layout_vertical[1]);

        widgets.search.draw(f, ctx, layout_vertical[0]);
        // Dont draw batch pane if empty
        match ctx.batch.is_empty() {
            true => widgets.results.draw(f, ctx, layout_vertical[1]),
            false => {
                widgets.results.draw(f, ctx, layout_horizontal[0]);
                widgets.batch.draw(f, ctx, layout_horizontal[1]);
            }
        }
        match ctx.mode {
            Mode::Category => widgets.category.draw(f, ctx, f.size()),
            Mode::Sort(_) => widgets.sort.draw(f, ctx, f.size()),
            Mode::Filter => widgets.filter.draw(f, ctx, f.size()),
            Mode::Theme => widgets.theme.draw(f, ctx, f.size()),
            Mode::Error => {
                // Get the oldest error first
                if let Some(error) = ctx.errors.pop_front() {
                    widgets.error.with_error(error);
                }
                widgets.error.draw(f, ctx, f.size());
            }
            Mode::Help => widgets.help.draw(f, ctx, f.size()),
            Mode::Page => widgets.page.draw(f, ctx, f.size()),
            Mode::User => widgets.user.draw(f, ctx, f.size()),
            Mode::Sources => widgets.sources.draw(f, ctx, f.size()),
            Mode::Clients => widgets.clients.draw(f, ctx, f.size()),
            Mode::KeyCombo(_) | Mode::Normal | Mode::Search | Mode::Loading(_) | Mode::Batch => {}
        }
    }

    fn on<B: Backend>(
        &mut self,
        evt: &Event,
        w: &mut Widgets,
        ctx: &mut Context,
        #[cfg(unix)] terminal: &mut Terminal<B>,
    ) {
        if let Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            modifiers,
            ..
        }) = evt
        {
            #[cfg(unix)]
            if let (KeyCode::Char('z'), &KeyModifiers::CONTROL) = (code, modifiers) {
                if let Err(e) = suspend_self(terminal) {
                    ctx.show_error(format!("Failed to suspend:\n{}", e));
                }
                // If we fail to continue the process, panic
                if continue_self(terminal).is_err() {
                    panic!("Failed to continue program");
                }
                return;
            }
            match ctx.mode.to_owned() {
                Mode::KeyCombo(keys) => {
                    ctx.last_key = keys.into_iter().collect::<String>();
                }
                _ => ctx.last_key = key_to_string(*code, *modifiers),
            };
        }
        match ctx.mode.to_owned() {
            Mode::Category => w.category.handle_event(ctx, evt),
            Mode::Sort(_) => w.sort.handle_event(ctx, evt),
            Mode::Normal => w.results.handle_event(ctx, evt),
            Mode::Batch => w.batch.handle_event(ctx, evt),
            Mode::Search => w.search.handle_event(ctx, evt),
            Mode::Filter => w.filter.handle_event(ctx, evt),
            Mode::Theme => w.theme.handle_event(ctx, evt),
            Mode::Error => w.error.handle_event(ctx, evt),
            Mode::Page => w.page.handle_event(ctx, evt),
            Mode::User => w.user.handle_event(ctx, evt),
            Mode::Help => w.help.handle_event(ctx, evt),
            Mode::Sources => w.sources.handle_event(ctx, evt),
            Mode::Clients => w.clients.handle_event(ctx, evt),
            Mode::KeyCombo(keys) => self.on_combo(w, ctx, keys, evt),
            Mode::Loading(_) => {}
        }
        if ctx.mode != Mode::Help {
            self.on_help(evt, ctx);
        }
    }

    fn on_help(&mut self, e: &Event, ctx: &mut Context) {
        if let Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            ..
        }) = e
        {
            match code {
                KeyCode::Char('?') if ctx.mode != Mode::Search => {
                    ctx.mode = Mode::Help;
                }
                KeyCode::F(1) => {
                    ctx.mode = Mode::Help;
                }
                _ => {}
            }
        }
    }

    fn get_help(&mut self, w: &mut Widgets, ctx: &Context) {
        let help = match ctx.mode {
            Mode::Category => CategoryPopup::get_help(),
            Mode::Sort(_) => SortPopup::get_help(),
            Mode::Normal => ResultsWidget::get_help(),
            Mode::Batch => BatchWidget::get_help(),
            Mode::Search => SearchWidget::get_help(),
            Mode::Filter => FilterPopup::get_help(),
            Mode::Theme => ThemePopup::get_help(),
            Mode::Page => PagePopup::get_help(),
            Mode::User => UserPopup::get_help(),
            Mode::Sources => SourcesPopup::get_help(),
            Mode::Clients => ClientsPopup::get_help(),
            Mode::Error => None,
            Mode::Help => None,
            Mode::KeyCombo(_) => None,
            Mode::Loading(_) => None,
        };
        if let Some(msg) = help {
            w.help.with_items(msg, ctx.mode.clone());
            w.help.table.select(0);
        }
    }

    fn on_combo(&mut self, w: &Widgets, ctx: &mut Context, mut keys: Vec<char>, e: &Event) {
        if let Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            ..
        }) = e
        {
            match code {
                // Only handle standard chars for now
                KeyCode::Char(c) => keys.push(*c),
                KeyCode::Esc => {
                    // Stop combo if esc
                    ctx.mode = Mode::Normal;
                    return;
                }
                _ => {}
            }
        }
        match keys[..] {
            ['y', c] => {
                let s = w.results.table.state.selected().unwrap_or(0);
                match w.results.table.items.get(s) {
                    Some(item) => {
                        let link = match c {
                            't' => item.torrent_link.to_owned(),
                            'm' => item.magnet_link.to_owned(),
                            'p' => item.post_link.to_owned(),
                            _ => return,
                        };
                        ctx.mode = Mode::Normal;
                        match clip::copy_to_clipboard(link.to_owned(), ctx.config.clipboard.clone())
                        {
                            Ok(_) => ctx.notify(format!("Copied \"{}\" to clipboard", link)),
                            Err(e) => ctx.show_error(e),
                        }
                    }
                    None => ctx.show_error(
                        "Failed to copy:\nFailed to get torrent link for selected item",
                    ),
                }
            }
            _ => ctx.mode = Mode::KeyCombo(keys.to_owned()),
        }
        ctx.last_key = keys.into_iter().collect();
    }
}
