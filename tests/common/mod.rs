use std::{error::Error, sync::Mutex};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use nyaa::{
    app::App,
    client::{Client, ClientConfig, DownloadResult},
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

pub struct TestSync;

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

static INPUTS_MX: Mutex<Vec<Event>> = Mutex::new(Vec::new());
// static QUERY_FN: Mutex<Option<QueryFn>> = Mutex::new(None);
// static WAIT_FOR_RESULTS: Mutex<bool> = Mutex::new(false);
// static RESULTS_FN: Mutex<Option<ResultsFn>> = Mutex::new(None);
// static RESULTS_SEND: Mutex<Option<Sender<bool>>> = Mutex::new(None);
// static RESULTS_RECV: Mutex<Option<Receiver<bool>>> = Mutex::new(None);

pub fn clear_events() {
    INPUTS_MX.lock().unwrap().clear();
    // *WAIT_FOR_RESULTS.lock().unwrap() = false;
    // let (send, recv) = broadcast::channel(1);
    // *RESULTS_SEND.lock().unwrap() = Some(send);
    // *RESULTS_RECV.lock().unwrap() = Some(recv);
}

// pub fn wait_for_results(wait: bool) {
//     *WAIT_FOR_RESULTS.lock().unwrap() = wait;
// }

// pub fn set_results(res: ResultsFn) {
//     *RESULTS.lock().unwrap() = Some(res);
// }

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

    pub fn set_events(&self) {
        INPUTS_MX.lock().unwrap().clone_from(&self.events);
    }
}

pub async fn run_app(w: u16, h: u16) -> Result<Terminal<TestBackend>, Box<dyn Error>> {
    let mut backend = TestBackend::new(w, h);
    let _ = backend.clear();
    let mut terminal = Terminal::new(backend)?;
    let _ = terminal.clear();

    let mut app = App::default();

    app.run_app::<_, TestSync>(&mut terminal).await?;
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
        // let queryfn = QUERY_FN
        //     .lock()
        //     .unwrap()
        //     .unwrap_or(|_, _, _, _, _, _, _| (Ok(Results::default()), false));
        // let (res, quit) = queryfn(loadtype, src, client, query, config, theme, date_format);
        // if let Some(res_fn) = *RESULTS_FN.lock().unwrap() {
        //     let res = res_fn(loadtype, src, client, query, config, theme, date_format);
        //     let _ = tx_res.send(Ok(res)).await;
        //     if let Some(sender) = RESULTS_SEND.lock().unwrap().as_mut() {
        //         let _ = sender.send(true);
        //     }
        //     return;
        // }
        // let res = (*RESULTS_FN.lock().unwrap())
        //     .map(|f| f(loadtype, src, client, query, config, theme, date_format))
        //     .unwrap_or_default();

        let _ = tx_res.send(Ok(Results::default())).await;
        // if let Some(sender) = RESULTS_SEND.lock().unwrap().as_mut() {
        //     // let _ = sender.send(true);
        // }
        // if quit {
        //     INPUTS_MX.lock().unwrap().push(Event::FocusLost);
        // }
        // for evt in inputs.into_iter() {
        //     let _ = tx_evt.send(evt).await;
        // }
        // let _ = tx_res
        //     .send(queryfn(
        //         tx_res,
        //         loadtype,
        //         src,
        //         client,
        //         query,
        //         config,
        //         theme,
        //         date_format,
        //     ))
        //     .await;

        // if let Some(func) = QUERY_FN.lock().unwrap().clone() {
        //     func(
        //         tx_res,
        //         loadtype,
        //         src,
        //         client,
        //         query,
        //         config,
        //         theme,
        //         date_format,
        //     );
        // } else {
        //     let _ = tx_res.send(Ok(Results::default())).await;
        // }
        // match QUERY_FN.lock().unwrap().clone() {
        //     Some(func) => func(
        //         tx_res,
        //         loadtype,
        //         src,
        //         client,
        //         query,
        //         config,
        //         theme,
        //         date_format,
        //     ),
        //     None => {} // None => {
        //                //     let _ = tx_res.send(Ok(Results::default())).await;
        //                // }
        // };
    }

    async fn read_event_loop(tx_evt: tokio::sync::mpsc::Sender<crossterm::event::Event>) {
        // if *WAIT_FOR_RESULTS.lock().unwrap() {
        //     let _ = RESULTS_RECV.lock().unwrap().as_mut().unwrap().recv().await;
        //     // let mut res2 = res.as_mut().unwrap();
        //     // let _ = res2.recv().await;
        //     // let recv = res.as_mut().unwrap();
        //     // tokio::select! {
        //     //     x = recv.recv() => {}
        //     // };
        // }
        let inputs = INPUTS_MX.lock().unwrap().clone();
        for evt in inputs.into_iter() {
            let _ = tx_evt.send(evt).await;
        }
        let _ = tx_evt.send(Event::FocusLost).await;
    }

    async fn download(
        _tx_dl: tokio::sync::mpsc::Sender<DownloadResult>,
        _batch: bool,
        _items: Vec<Item>,
        _config: ClientConfig,
        _rq_client: reqwest::Client,
        _client: Client,
    ) {
    }
}
