use std::error::Error;

use crossterm::event::{self, Event};
use tokio::sync::mpsc;

use crate::{
    app::LoadType,
    results::ResultTable,
    source::{SourceConfig, Sources},
    theme::Theme,
    widget::sort::SelectedSort,
};

// pub async fn read_event(kill: CancellationToken, tx_evt: mpsc::Sender<Event>) {
//     let task = tokio::spawn(read_event_loop(tx_evt));
//     kill.cancelled().await;
//     task.abort();
//     let _ = join!(task);
//     // tokio::select! {
//     //     biased;
//     //
//     //     _ = kill.cancelled() => (),
//     //     output = read_event_loop(tx_evt) => output,
//     // };
// }

pub struct SearchQuery {
    pub query: String,
    pub page: usize,
    pub category: usize,
    pub filter: usize,
    pub sort: SelectedSort,
    pub user: Option<String>,
    pub date_format: Option<String>,
}

pub async fn load_results(
    tx_res: mpsc::Sender<Result<ResultTable, Box<dyn Error + Send + Sync>>>,
    load_type: LoadType,
    src: Sources,
    client: reqwest::Client,
    search: SearchQuery,
    config: SourceConfig,
    theme: Theme,
) {
    let res = src.load(load_type, &client, search, config, theme).await;
    let _ = tx_res.send(res).await;
}

pub async fn read_event_loop(tx_evt: mpsc::Sender<Event>) {
    loop {
        if let Ok(evt) = event::read() {
            let _ = tx_evt.send(evt).await;
        }
    }
}
