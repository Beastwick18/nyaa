use std::{
    error::Error,
    fmt::Display,
    sync::Arc,
    time::{Duration, Instant},
};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use indexmap::IndexMap;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Position},
    Frame, Terminal,
};
use reqwest::cookie::Jar;
use tokio::{sync::mpsc, task::AbortHandle};

#[cfg(feature = "captcha")]
use crate::widget::captcha::CaptchaPopup;

use crate::{
    client::{Client, DownloadClientResult, SingleDownloadResult},
    clip::ClipboardManager,
    config::{Config, ConfigManager},
    results::Results,
    source::{
        nyaa_html::NyaaHtmlSource, request_client, Item, Source, SourceInfo, SourceResults, Sources,
    },
    sync::{EventSync, ReloadType, SearchQuery},
    theme::{self, Theme},
    util::conv::key_to_string,
    widget::{
        batch::BatchWidget,
        category::CategoryPopup,
        clients::ClientsPopup,
        filter::FilterPopup,
        help::HelpPopup,
        notifications::{Notification, NotificationWidget},
        page::PagePopup,
        results::ResultsWidget,
        search::SearchWidget,
        sort::{SortDir, SortPopup},
        sources::SourcesPopup,
        themes::ThemePopup,
        user::UserPopup,
        Widget,
    },
    widgets,
};

#[cfg(unix)]
use core::panic;
#[cfg(unix)]
use crossterm::event::KeyModifiers;

#[cfg(unix)]
use crate::util::term;

pub static APP_NAME: &str = "nyaa";

// To ensure that other events will get a chance to be received
static ANIMATE_SLEEP_MILLIS: u64 = 5;

#[derive(PartialEq, Clone)]
pub enum LoadType {
    Sourcing,
    Searching,
    SolvingCaptcha(String),
    Sorting,
    Filtering,
    Categorizing,
    Batching,
    Downloading,
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
    #[cfg(feature = "captcha")]
    Captcha,
}

widgets! {
    Widgets;
    batch: [Mode::Batch] => BatchWidget,
    search: [Mode::Search] => SearchWidget,
    results: [Mode::Normal] => ResultsWidget,
    notification: NotificationWidget,
    [popups]: {
        category: [Mode::Category]  => CategoryPopup,
        sort: [Mode::Sort(_)]  => SortPopup,
        filter: [Mode::Filter]  => FilterPopup,
        theme: [Mode::Theme]  => ThemePopup,
        sources: [Mode::Sources]  => SourcesPopup,
        clients: [Mode::Clients]  => ClientsPopup,
        page: [Mode::Page]  => PagePopup,
        user: [Mode::User] => UserPopup,
        help: [Mode::Help] => HelpPopup,
        #[cfg(feature = "captcha")]
        captcha: [Mode::Captcha] => CaptchaPopup,
    }
}

impl Display for LoadType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            LoadType::Sourcing => "Sourcing",
            LoadType::Searching => "Searching",
            LoadType::SolvingCaptcha(_) => "Solving",
            LoadType::Sorting => "Sorting",
            LoadType::Filtering => "Filtering",
            LoadType::Categorizing => "Categorizing",
            LoadType::Batching => "Downloading Batch",
            LoadType::Downloading => "Downloading",
        };
        write!(f, "{}", s)
    }
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
            #[cfg(feature = "captcha")]
            Mode::Captcha => "Captcha",
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
    //errors: Vec<String>,
    notifications: Vec<Notification>,
    failed_config_load: bool,
    should_quit: bool,
    should_dismiss_notifications: bool,
    should_save_config: bool,
    skip_reload: bool,
}

impl Context {
    pub fn notify_error<S: Display>(&mut self, msg: S) {
        self.notify(Notification::error(msg));
    }

    pub fn notify_info<S: Display>(&mut self, msg: S) {
        self.notify(Notification::info(msg));
    }

    pub fn notify_warn<S: Display>(&mut self, msg: S) {
        self.notify(Notification::warning(msg));
    }

    pub fn notify_success<S: Display>(&mut self, msg: S) {
        self.notify(Notification::success(msg));
    }

    pub fn notify(&mut self, notif: Notification) {
        self.notifications.push(notif);
    }

    pub fn dismiss_notifications(&mut self) {
        self.should_dismiss_notifications = true;
    }

    pub fn save_config(&mut self) -> Result<(), Box<dyn Error>> {
        self.should_save_config = true;
        self.skip_reload = true;
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
            skip_reload: false,
        }
    }
}

impl App {
    pub async fn run_app<B: Backend, S: EventSync + Clone, C: ConfigManager, const TEST: bool>(
        &mut self,
        terminal: &mut Terminal<B>,
        sync: S,
        config_manager: C,
    ) -> Result<(), Box<dyn Error>> {
        let ctx = &mut Context::default();

        let timer = tokio::time::sleep(Duration::from_millis(ANIMATE_SLEEP_MILLIS));
        tokio::pin!(timer);

        let (tx_res, mut rx_res) =
            mpsc::channel::<Result<SourceResults, Box<dyn Error + Send + Sync>>>(32);
        let (tx_evt, mut rx_evt) = mpsc::channel::<Event>(100);
        let (tx_dl, mut rx_dl) = mpsc::channel::<DownloadClientResult>(100);
        let (tx_cfg, mut rx_cfg) = mpsc::channel::<ReloadType>(1);

        tokio::task::spawn(sync.clone().read_event_loop(tx_evt));
        tokio::task::spawn(sync.clone().watch_config_loop(tx_cfg));

        match config_manager.load() {
            Ok(config) => {
                ctx.failed_config_load = false;
                if let Err(e) = config.full_apply(config_manager.path(), ctx, &mut self.widgets) {
                    ctx.notify_error(e);
                }
            }
            Err(e) => {
                ctx.notify_error(format!("Failed to load config:\n{}", e));
                if let Err(e) =
                    ctx.config
                        .clone()
                        .full_apply(config_manager.path(), ctx, &mut self.widgets)
                {
                    ctx.notify_error(e);
                }
            }
        }

        let jar = Arc::new(Jar::default());
        let source_rqclient =
            request_client(&jar, ctx.config.timeout, ctx.config.request_proxy.clone())?;
        // Don't use proxy for clients
        let client_rqclient = request_client(&jar, ctx.config.timeout, None)?;
        let mut last_load_abort: Option<AbortHandle> = None;
        let mut last_time: Option<Instant> = None;

        let (clipboard, err) = &mut if TEST {
            ClipboardManager::empty(ctx.config.clipboard.clone().unwrap_or_default())
        } else {
            ClipboardManager::new(ctx.config.clipboard.clone().unwrap_or_default())
        };
        if let Some(err) = err {
            ctx.notify_error(err);
        }

        while !ctx.should_quit {
            if ctx.should_save_config && ctx.config.save_config_on_change {
                if let Err(e) = config_manager.store(&ctx.config) {
                    ctx.notify_error(e);
                }
                ctx.should_save_config = false;
            }
            if !ctx.notifications.is_empty() {
                ctx.notifications
                    .clone()
                    .into_iter()
                    .for_each(|n| self.widgets.notification.add(n));
                ctx.notifications.clear();
            }
            //if !ctx.errors.is_empty() {
            //    if TEST {
            //        return Err(ctx.errors.join("\n\n").into());
            //    }
            //    ctx.errors
            //        .clone()
            //        .into_iter()
            //        .for_each(|n| self.widgets.notification.add(Notification::Error, n));
            //    ctx.errors.clear();
            //}
            if ctx.should_dismiss_notifications {
                self.widgets.notification.dismiss_all();
                ctx.should_dismiss_notifications = false;
            }
            if ctx.mode == Mode::Batch && ctx.batch.is_empty() {
                ctx.mode = Mode::Normal;
            }

            self.get_help(ctx);
            terminal.draw(|f| self.draw(ctx, f))?;
            if let Mode::Loading(load_type) = ctx.mode.clone() {
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
                                client_rqclient.clone(),
                                ctx.client,
                            ));
                            ctx.notify_info(format!("Downloading torrent with {}", ctx.client));
                        }
                        continue;
                    }
                    LoadType::Batching => {
                        tokio::spawn(sync.clone().download(
                            tx_dl.clone(),
                            true,
                            ctx.batch.clone(),
                            ctx.config.client.clone(),
                            client_rqclient.clone(),
                            ctx.client,
                        ));
                        ctx.notify_info(format!(
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

                ctx.load_type = Some(load_type.clone());

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
                    load_type.clone(),
                    ctx.src,
                    source_rqclient.clone(),
                    search,
                    ctx.config.sources.clone(),
                    ctx.theme.clone(),
                    ctx.config.clone().into(),
                ));
                last_load_abort = Some(task.abort_handle());
                continue; // Redraw
            }

            loop {
                tokio::select! {
                    biased;
                    Some(evt) = rx_evt.recv() => {
                        #[cfg(unix)]
                        self.on::<B, TEST>(&evt, ctx, clipboard, terminal);
                        #[cfg(not(unix))]
                        self.on::<B, TEST>(&evt, ctx, clipboard);

                        break;
                    },
                    () = &mut timer, if self.widgets.notification.is_animating() => {
                        timer.as_mut().reset(tokio::time::Instant::now() + Duration::from_millis(ANIMATE_SLEEP_MILLIS));
                        if let Ok(size) = terminal.size() {
                            let now = Instant::now();
                            ctx.deltatime = last_time.map(|l| (now - l).as_secs_f64()).unwrap_or(0.0);
                            last_time = Some(now);

                            if self.widgets.notification.update(ctx.deltatime, (Position::ORIGIN, size).into()) {
                                break;
                            }
                        } else {
                            break;
                        }
                    },
                    Some(rt) = rx_res.recv() => {
                        match rt {
                            Ok(SourceResults::Results(rt)) => {
                                self.widgets.results.reset();
                                ctx.results = rt;
                            }
                            #[cfg(feature = "captcha")]
                            Ok(SourceResults::Captcha(c)) => {
                                ctx.results = Results::default();
                                ctx.mode = Mode::Captcha;
                                self.widgets.captcha.image = Some(c);
                                self.widgets.captcha.input.clear();
                            }
                            Err(e) => {
                                // Clear results on error
                                ctx.results = Results::default();
                                ctx.notify_error(e);
                            },
                        }
                        ctx.load_type = None;
                        last_load_abort = None;
                        break;
                    },
                    Some(dl) = rx_dl.recv() => {
                        match dl {
                            DownloadClientResult::Single(sr) => {
                                match sr {
                                    SingleDownloadResult::Success(suc) => {
                                        ctx.notify(suc.msg);
                                    },
                                    SingleDownloadResult::Error(err) => {
                                        ctx.notify(err.msg);
                                    },
                                };
                            }
                            DownloadClientResult::Batch(br) => {
                                if !br.ids.is_empty() {
                                    ctx.notify(br.msg);
                                }
                                br.errors.into_iter().for_each(|e| ctx.notify(e));
                            }
                        }
                        break;
                    }
                    Some(notif) = rx_cfg.recv() => {
                        if ctx.skip_reload {
                            ctx.skip_reload = false;
                            continue;
                        }
                        match notif {
                            ReloadType::Config => {
                                match config_manager.load() {
                                    Ok(config) => {
                                        match config.partial_apply(ctx, &mut self.widgets) {
                                            Ok(()) => ctx.notify_info("Reloaded config".to_owned()),
                                            Err(e) => ctx.notify_error(e),
                                        }
                                    }
                                    Err(e) => ctx.notify_error(e),
                                }
                            },
                            ReloadType::Theme(t) => match theme::load_user_themes(ctx, config_manager.path()) {
                                Ok(()) => ctx.notify_info(format!("Reloaded theme \"{t}\"")),
                                Err(e) => ctx.notify_error(e)
                            },
                        }

                        break;
                    },
                    else => {
                        return Err("All channels closed".into());
                    }
                };
            }
            if !self.widgets.notification.is_animating() {
                last_time = None;
            }
        }
        Ok(())
    }

    pub fn draw(&mut self, ctx: &mut Context, f: &mut Frame) {
        let layout_vertical = Layout::new(
            Direction::Vertical,
            [Constraint::Length(3), Constraint::Min(1)],
        )
        .split(f.area());

        self.widgets.search.draw(f, ctx, layout_vertical[0]);
        // Dont draw batch pane if empty
        if ctx.batch.is_empty() {
            self.widgets.results.draw(f, ctx, layout_vertical[1]);
        } else {
            let layout_horizontal = Layout::new(
                Direction::Horizontal,
                match ctx.mode {
                    Mode::Batch | Mode::Help => [Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)],
                    _ => [Constraint::Ratio(3, 4), Constraint::Ratio(1, 4)],
                },
            )
            .split(layout_vertical[1]);
            self.widgets.results.draw(f, ctx, layout_horizontal[0]);
            self.widgets.batch.draw(f, ctx, layout_horizontal[1]);
        }
        self.widgets.draw_popups(ctx, f);
        self.widgets.notification.draw(f, ctx, f.area());
    }

    fn on<B: Backend, const TEST: bool>(
        &mut self,
        evt: &Event,
        ctx: &mut Context,
        clipboard: &mut ClipboardManager,
        #[cfg(unix)] terminal: &mut Terminal<B>,
    ) {
        if TEST && Event::FocusLost == *evt {
            ctx.quit();
            return;
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
                    ctx.notify_error(format!("Failed to suspend:\n{}", e));
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
            Mode::KeyCombo(keys) => self.on_combo(ctx, clipboard, keys, evt),
            Mode::Loading(_) => {}
            _ => self.widgets.handle_event(ctx, evt),
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
        let help = self.widgets.get_help(&ctx.mode);
        if let Some(msg) = help {
            self.widgets.help.with_items(msg, ctx.mode.clone());
            self.widgets.help.table.select(0);
        }
    }

    fn on_combo(
        &mut self,
        ctx: &mut Context,
        clipboard: &mut ClipboardManager,
        mut keys: String,
        e: &Event,
    ) {
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
                ctx.mode = Mode::Normal;
                match ctx.results.response.items.get(s).cloned() {
                    Some(item) => {
                        let link = match c {
                            't' => item.torrent_link,
                            'm' => {
                                if ctx.config.yank_full_magnet {
                                    match item.minimal_magnet_link() {
                                        Ok(magnet) => magnet,
                                        Err(e) => return ctx.notify_error(e),
                                    }
                                } else {
                                    item.magnet_link
                                }
                            }
                            'p' => item.post_link,
                            'i' => match item.extra.get("imdb").cloned() {
                                Some(imdb) => imdb,
                                None => return ctx.notify_error("No imdb ID found for this item."),
                            },
                            'n' => item.title,
                            _ => return,
                        };
                        match clipboard.try_copy(&link) {
                            Ok(()) => {
                                ctx.notify_success(format!("Copied \"{}\" to clipboard", link))
                            }
                            Err(e) => ctx.notify_error(e),
                        }
                    }
                    None if ['t', 'm', 'p', 'i', 'n'].contains(&c) => {
                        ctx.notify_error("Failed to copy:\nFailed to get item")
                    }
                    None => {}
                }
            }
            _ => ctx.mode = Mode::KeyCombo(keys),
        }
    }
}
