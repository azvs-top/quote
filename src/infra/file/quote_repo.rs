use std::path::PathBuf;
use async_trait::async_trait;
use crate::app::app_error::AppError;
use crate::quote::{Quote, QuotePort, QuoteQuery, QuoteQueryFilter};

pub struct QuoteRepoFile {
    path: PathBuf
}

impl QuoteRepoFile {
    pub fn new(path: PathBuf) -> Self {
        QuoteRepoFile { path }
    }

    fn apply_filter(&self,filter: &QuoteQueryFilter) {
        todo!()
    }
}

#[async_trait]
impl QuotePort for QuoteRepoFile {
    async fn get(&self, query: QuoteQuery) -> Result<Quote, AppError> {
        todo!()
    }

    async fn list(&self, query: QuoteQuery) -> Result<Vec<Quote>, AppError> {
        todo!()
    }
}