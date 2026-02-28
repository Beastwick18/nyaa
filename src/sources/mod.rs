use std::{fmt::Display, ops::Deref};

use async_trait::async_trait;
use derive_more::derive::Deref;
use enum_assoc::Assoc;
use nyaa::NyaaSource;

use crate::{
    result::Results,
    sources::query::{category::Category, sort::SortDirection},
};

pub mod nyaa;
pub mod query;

#[derive(Clone, Default)]
pub struct SourceTaskState {
    pub search: String,
    pub filter_idx: usize,
    pub sort_idx: usize,
    pub sort_dir: SortDirection,
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
    async fn search(&self, state: SourceTaskState) -> Result<Results, SourceError>;

    fn filters(&self) -> Vec<String>;
    fn sorts(&self) -> Vec<String>;
    fn categories(&self) -> Vec<Category>;

    // fn set_filter(&mut self, index: usize);
    // fn set_sort(&mut self, index: usize);
    // fn set_category(&mut self, index: usize);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceError(pub String);

impl Deref for SourceError {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<S: Display> From<S> for SourceError {
    fn from(value: S) -> Self {
        SourceError(value.to_string())
    }
}

pub struct SourceTaskRunner;

impl SourceTaskRunner {
    pub async fn run(
        source: Source,
        state: SourceTaskState,
    ) -> Result<Option<Results>, SourceError> {
        let src = source.source();
        Ok(Some(src.search(state).await?))
    }
}
