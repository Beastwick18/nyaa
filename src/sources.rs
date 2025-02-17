use enum_assoc::Assoc;
use nyaa::NyaaSource;

use crate::result::Results;

pub mod nyaa;

#[derive(Assoc, Clone, Copy)]
#[func(pub const fn source(&self) -> &dyn SourceTask)]
pub enum Source {
    #[assoc(source = &NyaaSource)]
    Nyaa,
}

pub trait SourceTask {
    fn search(&self, query: String) -> Results;
}

pub struct SourceTaskRunner;

impl SourceTaskRunner {
    pub async fn run(source: Source, query: String) -> Option<Results> {
        let src = source.source();
        Some(src.search(query))
    }
}
