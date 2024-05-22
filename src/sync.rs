use std::error::Error;

use crossterm::event::{self, Event};
use tokio::sync::mpsc;

use crate::{
    app::LoadType,
    results::Results,
    source::{SourceConfig, Sources},
    theme::Theme,
    widget::sort::SelectedSort,
};

#[derive(Clone, Default)]
pub struct SearchQuery {
    pub query: String,
    pub page: usize,
    pub category: usize,
    pub filter: usize,
    pub sort: SelectedSort,
    pub user: Option<String>,
}

#[allow(clippy::too_many_arguments)]
pub async fn load_results(
    tx_res: mpsc::Sender<Result<Results, Box<dyn Error + Send + Sync>>>,
    load_type: LoadType,
    src: Sources,
    client: reqwest::Client,
    search: SearchQuery,
    config: SourceConfig,
    theme: Theme,
    date_format: Option<String>,
) {
    let res = src
        .load(load_type, &client, &search, &config, date_format)
        .await;
    let fmt = res.map(|res| {
        Results::new(
            search.clone(),
            res.clone(),
            src.format_table(&res.items, &search, &config, &theme),
        )
    });
    let _ = tx_res.send(fmt).await;
}

pub async fn read_event_loop(tx_evt: mpsc::Sender<Event>) {
    loop {
        if let Ok(evt) = event::read() {
            let _ = tx_evt.send(evt).await;
        }
    }
}
