use std::{error::Error, fmt::Display, time::Instant};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use indexmap::IndexMap;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    Frame, Terminal,
};
use tokio::{sync::mpsc, task::AbortHandle};

use crate::{
    client::{Client, DownloadResult},
    clip,
    config::{Config, ConfigManager},
    results::Results,
    source::{nyaa_html::NyaaHtmlSource, request_client, Item, Source, SourceInfo, Sources},
    sync::{EventSync, SearchQuery},
    theme::{self, Theme},
    util::conv::key_to_string,
    widget::{
        batch::BatchWidget,
        category::CategoryPopup,
        clients::ClientsPopup,
        filter::FilterPopup,
        help::HelpPopup,
        notifications::NotificationWidget,
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
    KeyCombo(String),
    Search,
    Category,
    Sort(SortDir),
    Batch,
    Filter,
    Theme,
    Sources,
    Clients,
    Page,
    User,
    Help,
}

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Mode::Normal | Mode::KeyCombo(_) => "Normal",
            Mode::Batch => "Batch",
            Mode::Search => "Search",
            Mode::Category => "Category",
            Mode::Sort(_) => "Sort",
            Mode::Filter => "Filter",
            Mode::Theme => "Theme",
            Mode::Sources => "Sources",
            Mode::Clients => "Clients",
            Mode::Loading(_) => "Loading",
            Mode::Page => "Page",
            Mode::User => "User",
            Mode::Help => "Help",
        }
        .to_owned();
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
    pub page: usize,
    pub user: Option<String>,
    pub src: Sources,
    pub client: Client,
    pub batch: Vec<Item>,
    pub last_key: String,
    pub results: Results,
    pub deltatime: f64,
    errors: Vec<String>,
    notifications: Vec<String>,
    failed_config_load: bool,
    should_quit: bool,
    should_dismiss_notifications: bool,
    should_save_config: bool,
}

impl Context {
    pub fn show_error<S: Display>(&mut self, error: S) {
        self.errors.push(error.to_string());
    }

    pub fn notify<S: Display>(&mut self, msg: S) {
        self.notifications.push(msg.to_string());
    }

    pub fn dismiss_notifications(&mut self) {
        self.should_dismiss_notifications = true;
    }

    pub fn save_config(&mut self) -> Result<(), Box<dyn Error>> {
        self.should_save_config = true;
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
            errors: Vec::new(),
            notifications: Vec::new(),
            page: 1,
            user: None,
            src: Sources::Nyaa,
            client: Client::Cmd,
            batch: vec![],
            last_key: "".to_owned(),
            results: Results::default(),
            deltatime: 0.0,
            failed_config_load: true,
            should_quit: false,
            should_dismiss_notifications: false,
            should_save_config: false,
        }
    }
}

#[derive(Default)]
pub struct Widgets {
    pub notification: NotificationWidget,
    pub batch: BatchWidget,
    pub category: CategoryPopup,
    pub sort: SortPopup,
    pub filter: FilterPopup,
    pub theme: ThemePopup,
    pub sources: SourcesPopup,
    pub clients: ClientsPopup,
    pub search: SearchWidget,
    pub results: ResultsWidget,
    pub page: PagePopup,
    pub user: UserPopup,
    pub help: HelpPopup,
}

impl App {
    pub async fn run_app<B: Backend, S: EventSync + Clone, C: ConfigManager, const TEST: bool>(
        &mut self,
        terminal: &mut Terminal<B>,
        sync: S,
    ) -> Result<(), Box<dyn Error>> {
        let ctx = &mut Context::default();

        let (tx_res, mut rx_res) =
            mpsc::channel::<Result<Results, Box<dyn Error + Send + Sync>>>(32);
        let (tx_evt, mut rx_evt) = mpsc::channel::<Event>(100);
        let (tx_dl, mut rx_dl) = mpsc::channel::<DownloadResult>(100);

        tokio::task::spawn(sync.clone().read_event_loop(tx_evt));

        match C::load() {
            Ok(config) => {
                ctx.failed_config_load = false;
                if let Err(e) = config.apply(ctx, &mut self.widgets) {
                    ctx.show_error(e);
                } else if let Err(e) = ctx.save_config() {
                    ctx.show_error(e);
                }
            }
            Err(e) => {
                ctx.show_error(format!("Failed to load config:\n{}", e));
                if let Err(e) = ctx.config.clone().apply(ctx, &mut self.widgets) {
                    ctx.show_error(e);
                }
            }
        }

        let client = request_client(ctx)?;
        let mut last_load_abort: Option<AbortHandle> = None;
        let mut last_time: Option<Instant> = None;

        while !ctx.should_quit {
            if ctx.should_save_config {
                if let Err(e) = C::store(&ctx.config) {
                    ctx.show_error(e);
                }
            }
            if !ctx.notifications.is_empty() {
                ctx.notifications
                    .clone()
                    .into_iter()
                    .for_each(|n| self.widgets.notification.add_notification(n));
                ctx.notifications.clear();
            }
            if !ctx.errors.is_empty() {
                ctx.errors
                    .clone()
                    .into_iter()
                    .for_each(|n| self.widgets.notification.add_error(n));
                ctx.errors.clear();
            }
            if ctx.should_dismiss_notifications {
                self.widgets.notification.dismiss_all();
                ctx.should_dismiss_notifications = false;
            }
            if ctx.mode == Mode::Batch && ctx.batch.is_empty() {
                ctx.mode = Mode::Normal;
            }

            self.get_help(ctx);
            terminal.draw(|f| self.draw(ctx, f))?;
            if let Mode::Loading(load_type) = ctx.mode {
                ctx.mode = Mode::Normal;
                match load_type {
                    LoadType::Downloading => {
                        if let Some(i) = self
                            .widgets
                            .results
                            .table
                            .selected()
                            .and_then(|i| ctx.results.response.items.get(i))
                        {
                            tokio::spawn(sync.clone().download(
                                tx_dl.clone(),
                                false,
                                vec![i.to_owned()],
                                ctx.config.client.clone(),
                                client.clone(),
                                ctx.client,
                            ));
                            ctx.notify(format!("Downloading torrent with {}", ctx.client));
                        }
                        continue;
                    }
                    LoadType::Batching => {
                        tokio::spawn(sync.clone().download(
                            tx_dl.clone(),
                            true,
                            ctx.batch.clone(),
                            ctx.config.client.clone(),
                            client.clone(),
                            ctx.client,
                        ));
                        ctx.notify(format!(
                            "Downloading {} torrents with {}",
                            ctx.batch.len(),
                            ctx.client
                        ));
                        continue;
                    }
                    LoadType::Sourcing => {
                        // On sourcing, update info, reset things like category, etc.
                        ctx.src.apply(ctx, &mut self.widgets);
                    }
                    _ => {}
                }

                ctx.load_type = Some(load_type);

                if let Some(handle) = last_load_abort.as_ref() {
                    handle.abort();
                }

                let search = SearchQuery {
                    query: self.widgets.search.input.input.clone(),
                    page: ctx.page,
                    category: self.widgets.category.selected,
                    filter: self.widgets.filter.selected,
                    sort: self.widgets.sort.selected,
                    user: ctx.user.clone(),
                };

                let task = tokio::spawn(sync.clone().load_results(
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

            loop {
                tokio::select! {
                    biased;
                    Some(evt) = rx_evt.recv() => {
                        #[cfg(unix)]
                        self.on::<B, TEST>(&evt, ctx, terminal);
                        #[cfg(not(unix))]
                        self.on::<B, TEST>(&evt, ctx);

                        last_time = None;
                        break;
                    },
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
                        last_time = None;
                        break;
                    },
                    Some(dl) = rx_dl.recv() => {
                        if dl.batch {
                            for id in dl.success_ids.iter() {
                                ctx.batch.retain(|i| i.id.ne(id));
                            }
                        }
                        if !dl.success_ids.is_empty() {
                            if let Some(notif) = dl.success_msg {
                                ctx.notify(notif);
                            }
                        }
                        for e in dl.errors.iter() {
                            ctx.show_error(e)
                        }
                        break;
                    }
                    _ = async{}, if self.widgets.notification.is_animating() => {
                        if let Ok(size) = terminal.size() {
                            let now = Instant::now();
                            ctx.deltatime = (now - last_time.unwrap_or(now)).as_secs_f64();
                            last_time = Some(now);

                            if self.widgets.notification.update(ctx.deltatime, size) {
                                break;
                            }
                        } else {
                            break;
                        }
                    },
                    else => {
                        break;
                    }
                };
            }
        }
        Ok(())
    }

    pub fn draw(&mut self, ctx: &mut Context, f: &mut Frame) {
        let layout_vertical = Layout::new(
            Direction::Vertical,
            [Constraint::Length(3), Constraint::Min(1)],
        )
        .split(f.size());

        self.widgets.search.draw(f, ctx, layout_vertical[0]);
        // Dont draw batch pane if empty
        match ctx.batch.is_empty() {
            true => self.widgets.results.draw(f, ctx, layout_vertical[1]),
            false => {
                let layout_horizontal = Layout::new(
                    Direction::Horizontal,
                    match ctx.mode {
                        Mode::Batch => [Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)],
                        _ => [Constraint::Ratio(3, 4), Constraint::Ratio(1, 4)],
                    },
                )
                .split(layout_vertical[1]);
                self.widgets.results.draw(f, ctx, layout_horizontal[0]);
                self.widgets.batch.draw(f, ctx, layout_horizontal[1]);
            }
        }
        match ctx.mode {
            Mode::Category => self.widgets.category.draw(f, ctx, f.size()),
            Mode::Sort(_) => self.widgets.sort.draw(f, ctx, f.size()),
            Mode::Filter => self.widgets.filter.draw(f, ctx, f.size()),
            Mode::Theme => self.widgets.theme.draw(f, ctx, f.size()),
            Mode::Help => self.widgets.help.draw(f, ctx, f.size()),
            Mode::Page => self.widgets.page.draw(f, ctx, f.size()),
            Mode::User => self.widgets.user.draw(f, ctx, f.size()),
            Mode::Sources => self.widgets.sources.draw(f, ctx, f.size()),
            Mode::Clients => self.widgets.clients.draw(f, ctx, f.size()),
            Mode::KeyCombo(_) | Mode::Normal | Mode::Search | Mode::Loading(_) | Mode::Batch => {}
        }
        self.widgets.notification.draw(f, ctx, f.size());
    }

    fn on<B: Backend, const TEST: bool>(
        &mut self,
        evt: &Event,
        ctx: &mut Context,
        #[cfg(unix)] terminal: &mut Terminal<B>,
    ) {
        if TEST {
            if let Event::FocusLost = evt {
                ctx.quit();
            }
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
                    ctx.last_key = keys;
                }
                _ => ctx.last_key = key_to_string(*code, *modifiers),
            };
        }
        match ctx.mode.to_owned() {
            Mode::Category => self.widgets.category.handle_event(ctx, evt),
            Mode::Sort(_) => self.widgets.sort.handle_event(ctx, evt),
            Mode::Normal => self.widgets.results.handle_event(ctx, evt),
            Mode::Batch => self.widgets.batch.handle_event(ctx, evt),
            Mode::Search => self.widgets.search.handle_event(ctx, evt),
            Mode::Filter => self.widgets.filter.handle_event(ctx, evt),
            Mode::Theme => self.widgets.theme.handle_event(ctx, evt),
            Mode::Page => self.widgets.page.handle_event(ctx, evt),
            Mode::User => self.widgets.user.handle_event(ctx, evt),
            Mode::Help => self.widgets.help.handle_event(ctx, evt),
            Mode::Sources => self.widgets.sources.handle_event(ctx, evt),
            Mode::Clients => self.widgets.clients.handle_event(ctx, evt),
            Mode::KeyCombo(keys) => self.on_combo(ctx, keys, evt),
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

    fn get_help(&mut self, ctx: &Context) {
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
            Mode::Help => None,
            Mode::KeyCombo(_) => None,
            Mode::Loading(_) => None,
        };
        if let Some(msg) = help {
            self.widgets.help.with_items(msg, ctx.mode.clone());
            self.widgets.help.table.select(0);
        }
    }

    fn on_combo(&mut self, ctx: &mut Context, mut keys: String, e: &Event) {
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
        ctx.last_key.clone_from(&keys);
        match keys.chars().collect::<Vec<char>>()[..] {
            ['y', c] => {
                let s = self.widgets.results.table.state.selected().unwrap_or(0);
                match ctx.results.response.items.get(s) {
                    Some(item) => {
                        let link = match c {
                            't' => item.torrent_link.to_owned(),
                            'm' => item.magnet_link.to_owned(),
                            'p' => item.post_link.to_owned(),
                            _ => return,
                        };
                        match clip::copy_to_clipboard(link.to_owned(), ctx.config.clipboard.clone())
                        {
                            Ok(_) => ctx.notify(format!("Copied \"{}\" to clipboard", link)),
                            Err(e) => ctx.show_error(e),
                        }
                    }
                    None if ['t', 'm', 'p'].contains(&c) => {
                        ctx.show_error("Failed to copy:\nFailed to get item")
                    }
                    None => {}
                }
                ctx.mode = Mode::Normal;
            }
            _ => ctx.mode = Mode::KeyCombo(keys),
        }
    }
}
