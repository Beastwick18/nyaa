use std::fmt::Display;

use async_trait::async_trait;
use derive_more::derive::Deref;
use enum_assoc::Assoc;
use nyaa::NyaaSource;

use crate::{result::Results, sources::query::category::Category};

pub mod nyaa;
pub mod query;

#[derive(Clone, Copy, Default)]
pub struct SourceTaskState {
    pub filter_idx: usize,
    pub sort_idx: usize,
    pub category_group_idx: usize,
    pub category_idx: usize,
    pub page: usize,
}

#[derive(Assoc, Clone, Copy)]
#[func(pub fn source(&self) -> &impl SourceTask)]
pub enum Source {
    #[assoc(source = &NyaaSource)]
    Nyaa,
}

#[async_trait]
pub trait SourceTask {
    async fn search(&self, query: String, state: SourceTaskState) -> Result<Results, SourceError>;

    fn filters(&self) -> Vec<String>;
    fn sorts(&self) -> Vec<String>;
    fn categories(&self) -> Vec<Category>;

    // fn set_filter(&mut self, index: usize);
    // fn set_sort(&mut self, index: usize);
    // fn set_category(&mut self, index: usize);
}

#[derive(Deref, Debug, Clone, PartialEq, Eq)]
pub struct SourceError(pub String);

impl Display for SourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct SourceTaskRunner;

impl SourceTaskRunner {
    pub async fn run(
        source: Source,
        query: String,
        state: SourceTaskState,
    ) -> Result<Option<Results>, SourceError> {
        let src = source.source();
        Ok(Some(src.search(query, state).await?))
    }
}
