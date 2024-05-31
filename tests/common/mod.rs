use std::{error::Error, path::PathBuf};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use nyaa::{
    app::App,
    client::{Client, ClientConfig, DownloadResult},
    config::{Config, ConfigManager},
    results::Results,
    source::Item,
    sync::EventSync,
};
use ratatui::{
    backend::{Backend as _, TestBackend},
    buffer::Buffer,
    style::Style,
    Terminal,
};

#[derive(Clone)]
pub struct TestSync {
    events: Vec<Event>,
}

pub struct TestConfig;

pub type QueryFn = fn(
    nyaa::app::LoadType,
    nyaa::source::Sources,
    reqwest::Client,
    nyaa::sync::SearchQuery,
    nyaa::source::SourceConfig,
    nyaa::theme::Theme,
    Option<String>,
) -> (Result<Results, Box<dyn Error + Send + Sync>>, bool);

type ResultsFn = fn(
    nyaa::app::LoadType,
    nyaa::source::Sources,
    reqwest::Client,
    nyaa::sync::SearchQuery,
    nyaa::source::SourceConfig,
    nyaa::theme::Theme,
    Option<String>,
) -> Results;

pub struct EventBuilder {
    events: Vec<Event>,
}

impl EventBuilder {
    pub fn new() -> Self {
        EventBuilder { events: Vec::new() }
    }

    pub fn string<S: Into<String>>(&mut self, string: S) -> &mut Self {
        let evts = Into::<String>::into(string)
            .chars()
            .map(|c| {
                let modif = match c.is_uppercase() || "~!@#$%^&*()_+{}|:\"<>?".contains(c) {
                    true => KeyModifiers::SHIFT,
                    false => KeyModifiers::NONE,
                };
                Event::Key(KeyEvent::new(KeyCode::Char(c), modif))
            })
            .collect::<Vec<Event>>();
        self.events.extend(evts);
        self
    }

    pub fn quit(&mut self) -> &mut Self {
        self.push(Event::FocusLost)
    }

    pub fn esc(&mut self) -> &mut Self {
        self.key(KeyCode::Esc)
    }

    pub fn enter(&mut self) -> &mut Self {
        self.key(KeyCode::Enter)
    }

    pub fn tab(&mut self) -> &mut Self {
        self.key(KeyCode::Tab)
    }

    pub fn back_tab(&mut self) -> &mut Self {
        self.key_mod(KeyCode::BackTab, KeyModifiers::SHIFT)
    }

    pub fn push(&mut self, evt: Event) -> &mut Self {
        self.events.push(evt);
        self
    }

    pub fn key(&mut self, key: KeyCode) -> &mut Self {
        self.key_mod(key, KeyModifiers::NONE)
    }

    pub fn key_mod(&mut self, key: KeyCode, modifier: KeyModifiers) -> &mut Self {
        self.events.push(Event::Key(KeyEvent::new(key, modifier)));
        self
    }

    pub fn build(&mut self) -> TestSync {
        TestSync {
            events: self.events.clone(),
        }
    }
}

pub async fn run_app<S: EventSync + Clone>(
    sync: S,
    w: u16,
    h: u16,
) -> Result<Terminal<TestBackend>, Box<dyn Error>> {
    let mut backend = TestBackend::new(w, h);
    let _ = backend.clear();
    let mut terminal = Terminal::new(backend)?;
    let _ = terminal.clear();

    let mut app = App::default();

    app.run_app::<_, S, TestConfig, true>(&mut terminal, sync)
        .await?;
    Ok(terminal)
}

pub fn reset_buffer(terminal: &Terminal<TestBackend>) -> Buffer {
    let area = terminal.size().unwrap();
    let mut buf = terminal.backend().buffer().clone();
    buf.set_style(area, Style::reset());
    buf
}

pub fn print_buffer(buf: &Buffer) {
    println!();
    let mut len = 0;
    for cell in buf.content.clone().into_iter() {
        let sym = cell.symbol();
        let sym = sym.replace('\n', "");
        print!("{}", sym);
        len += 1;
        if len >= buf.area.width {
            println!();
            len = 0;
        }
    }
}

impl EventSync for TestSync {
    async fn load_results(
        self,
        tx_res: tokio::sync::mpsc::Sender<
            Result<nyaa::results::Results, Box<dyn std::error::Error + Send + Sync>>,
        >,
        _loadtype: nyaa::app::LoadType,
        _src: nyaa::source::Sources,
        _client: reqwest::Client,
        _query: nyaa::sync::SearchQuery,
        _config: nyaa::source::SourceConfig,
        _theme: nyaa::theme::Theme,
        _date_format: Option<String>,
    ) {
        let _ = tx_res.send(Ok(Results::default())).await;
    }

    async fn read_event_loop(self, tx_evt: tokio::sync::mpsc::Sender<crossterm::event::Event>) {
        for evt in self.events.into_iter() {
            let _ = tx_evt.send(evt).await;
        }
        let _ = tx_evt.send(Event::FocusLost).await;
    }

    async fn download(
        self,
        _tx_dl: tokio::sync::mpsc::Sender<DownloadResult>,
        _batch: bool,
        _items: Vec<Item>,
        _config: ClientConfig,
        _rq_client: reqwest::Client,
        _client: Client,
    ) {
    }
}

impl ConfigManager for TestConfig {
    fn load() -> Result<nyaa::config::Config, Box<dyn Error>> {
        Ok(Config::default())
    }

    fn store(_cfg: &nyaa::config::Config) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn path() -> Result<std::path::PathBuf, Box<dyn Error>> {
        Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/config"))
    }
}
