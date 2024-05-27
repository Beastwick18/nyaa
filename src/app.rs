use std::{collections::VecDeque, error::Error, fmt::Display};

use confy::ConfyError;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use indexmap::IndexMap;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    Frame, Terminal,
};
use tokio::{sync::mpsc, task::AbortHandle};

use crate::{
    client::Client,
    clip,
    config::{Config, CONFIG_FILE},
    results::Results,
    source::{nyaa_html::NyaaHtmlSource, request_client, Item, Source, SourceInfo, Sources},
    sync::{EventSync, SearchQuery},
    theme::{self, Theme},
    util::conv::key_to_string,
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
use crate::util::term;

pub static APP_NAME: &str = "nyaa";

#[derive(PartialEq, Clone, Copy)]
pub enum LoadType {
    Sourcing,
    Searching,
    Sorting,
    Filtering,
    Categorizing,
    Batching,
    Downloading,
}

impl Display for LoadType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            LoadType::Sourcing => "Sourcing",
            LoadType::Searching => "Searching",
            LoadType::Sorting => "Sorting",
            LoadType::Filtering => "Filtering",
            LoadType::Categorizing => "Categorizing",
            LoadType::Batching => "Downloading Batch",
            LoadType::Downloading => "Downloading",
        };
        write!(f, "{}", s)
    }
}

#[derive(PartialEq, Clone)]
pub enum Mode {
    Normal,
    Loading(LoadType),
    KeyCombo(Vec<char>),
    Search,
    Category,
    Sort(SortDir),
    Batch,
    Filter,
    Theme,
    Sources,
    Clients,
    Error,
    Page,
    User,
    Help,
}

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
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
        };
        write!(f, "{}", s)
    }
}

#[derive(Default)]
pub struct App {
    pub widgets: Widgets,
}

#[derive(Clone)]
pub struct Context {
    pub mode: Mode,
    pub load_type: Option<LoadType>,
    pub themes: IndexMap<String, Theme>,
    pub src_info: SourceInfo,
    pub theme: Theme,
    pub config: Config,
    pub errors: VecDeque<String>,
    pub notification: Option<String>,
    pub page: usize,
    pub user: Option<String>,
    pub src: Sources,
    pub client: Client,
    pub batch: Vec<Item>,
    pub last_key: String,
    pub results: Results,
    failed_config_load: bool,
    should_quit: bool,
}

impl Context {
    pub fn show_error<S: Display>(&mut self, error: S) {
        self.errors.push_back(error.to_string());
    }

    pub fn notify<S: Display>(&mut self, notification: S) {
        self.notification = Some(notification.to_string());
    }

    pub fn save_config(&mut self) -> Result<(), ConfyError> {
        if !self.failed_config_load {
            return confy::store::<&Config>(APP_NAME, CONFIG_FILE, &self.config);
        }
        Ok(())
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}

impl Default for Context {
    fn default() -> Self {
        Context {
            mode: Mode::Loading(LoadType::Searching),
            load_type: None,
            themes: theme::default_themes(),
            src_info: NyaaHtmlSource::info(),
            theme: Theme::default(),
            config: Config::default(),
            errors: VecDeque::new(),
            notification: None,
            page: 1,
            user: None,
            src: Sources::Nyaa,
            client: Client::Cmd,
            batch: vec![],
            last_key: "".to_owned(),
            should_quit: false,
            failed_config_load: true,
            results: Results::default(),
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
    pub async fn run_app<B: Backend, S: EventSync>(
        &mut self,
        terminal: &mut Terminal<B>,
    ) -> Result<(), Box<dyn Error>> {
        let w = &mut Widgets::default();
        let ctx = &mut Context::default();

        let (tx_res, mut rx_res) =
            mpsc::channel::<Result<Results, Box<dyn Error + Send + Sync>>>(32);
        let (tx_evt, mut rx_evt) = mpsc::channel::<Event>(100);

        tokio::task::spawn(S::read_event_loop(tx_evt));

        match Config::load() {
            Ok(config) => {
                ctx.failed_config_load = false;
                if let Err(e) = config.apply(ctx, w) {
                    ctx.show_error(e);
                } else if let Err(e) = ctx.save_config() {
                    ctx.show_error(e);
                }
            }
            Err(e) => {
                ctx.show_error(format!("Failed to load config:\n{}", e));
                if let Err(e) = ctx.config.clone().apply(ctx, w) {
                    ctx.show_error(e);
                }
            }
        }

        let client = request_client(ctx)?;
        let mut last_load_abort: Option<AbortHandle> = None;

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
                ctx.load_type = Some(load_type);
                ctx.mode = Mode::Normal;
                match load_type {
                    LoadType::Downloading => {
                        if let Some(i) = w
                            .results
                            .table
                            .selected()
                            .and_then(|i| ctx.results.response.items.get(i))
                        {
                            ctx.client.clone().download(i.to_owned(), ctx).await;
                        }
                        continue;
                    }
                    LoadType::Batching => {
                        ctx.client
                            .clone()
                            .batch_download(ctx.batch.clone(), ctx)
                            .await;
                        continue;
                    }
                    LoadType::Sourcing => {
                        // On sourcing, update info, reset things like category, etc.
                        ctx.src.apply(ctx, w);
                    }
                    _ => {}
                }

                if let Some(handle) = last_load_abort.as_ref() {
                    handle.abort();
                }

                let search = SearchQuery {
                    query: w.search.input.input.clone(),
                    page: ctx.page,
                    category: w.category.selected,
                    filter: w.filter.selected,
                    sort: w.sort.selected,
                    user: ctx.user.clone(),
                };

                let task = tokio::spawn(S::load_results(
                    tx_res.clone(),
                    load_type,
                    ctx.src,
                    client.clone(),
                    search,
                    ctx.config.sources.clone(),
                    ctx.theme.clone(),
                    ctx.config.date_format.clone(),
                ));
                last_load_abort = Some(task.abort_handle());
                continue; // Redraw
            }

            tokio::select! {
                Some(rt) = rx_res.recv() => {
                    match rt {
                        Ok(rt) => ctx.results = rt,
                        Err(e) => {
                            // Clear results on error
                            ctx.results = Results::default();
                            ctx.show_error(e);
                        },
                    }
                    ctx.load_type = None;
                    last_load_abort = None;
                },
                Some(evt) = rx_evt.recv() => {
                    #[cfg(unix)]
                    self.on(&evt, w, ctx, terminal);
                    #[cfg(not(unix))]
                    self.on::<B>(&evt, w, ctx);
                },
                else => {
                    break;
                }
            };
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
        #[cfg(feature = "integration-test")]
        if let Event::FocusLost = evt {
            ctx.quit();
        }

        if let Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            modifiers,
            ..
        }) = evt
        {
            #[cfg(unix)]
            if let (KeyCode::Char('z'), &KeyModifiers::CONTROL) = (code, modifiers) {
                if let Err(e) = term::suspend_self(terminal) {
                    ctx.show_error(format!("Failed to suspend:\n{}", e));
                }
                // If we fail to continue the process, panic
                if let Err(e) = term::continue_self(terminal) {
                    panic!("Failed to continue program:\n{}", e);
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
                match ctx.results.response.items.get(s) {
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
