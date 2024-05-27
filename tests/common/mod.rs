use std::{error::Error, sync::Mutex};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use nyaa::{app::App, results::Results, sync::EventSync};
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

static INPUTS_MX: Mutex<Vec<Event>> = Mutex::new(Vec::new());
static QUERY_FN: Mutex<Option<QueryFn>> = Mutex::new(None);

pub fn clear_events() {
    INPUTS_MX.lock().unwrap().clear();
}

// pub fn set_query_fn(queryfn: QueryFn) {
//     QUERY_FN.lock().unwrap().clone_from(&Some(queryfn));
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
            .map(|c| Event::Key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)))
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
        loadtype: nyaa::app::LoadType,
        src: nyaa::source::Sources,
        client: reqwest::Client,
        query: nyaa::sync::SearchQuery,
        config: nyaa::source::SourceConfig,
        theme: nyaa::theme::Theme,
        date_format: Option<String>,
    ) {
        let queryfn = QUERY_FN
            .lock()
            .unwrap()
            .unwrap_or(|_, _, _, _, _, _, _| (Ok(Results::default()), false));
        let (res, quit) = queryfn(loadtype, src, client, query, config, theme, date_format);
        let _ = tx_res.send(res).await;
        if quit {
            INPUTS_MX.lock().unwrap().push(Event::FocusLost);
        }
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
        let inputs = INPUTS_MX.lock().unwrap().clone();
        for evt in inputs.into_iter() {
            let _ = tx_evt.send(evt).await;
        }
        let _ = tx_evt.send(Event::FocusLost).await;
    }
}
