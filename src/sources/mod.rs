use std::fmt::Display;

use derive_more::derive::Deref;
use enum_assoc::Assoc;
use nyaa::NyaaSource;

use crate::result::Results;

pub mod nyaa;

#[derive(Assoc, Clone, Copy)]
#[func(pub const fn source(&self) -> &impl SourceTask)]
pub enum Source {
    #[assoc(source = &NyaaSource)]
    Nyaa,
}

pub trait SourceTask {
    fn search(
        &self,
        query: String,
    ) -> impl std::future::Future<Output = Result<Results, SourceError>> + std::marker::Send;
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
    pub async fn run(source: Source, query: String) -> Result<Option<Results>, SourceError> {
        let src = source.source();
        Ok(Some(src.search(query).await?))
    }
}
