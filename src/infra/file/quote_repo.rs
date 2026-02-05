use std::path::PathBuf;
use async_trait::async_trait;
use crate::app::app_error::AppError;
use crate::quote::{Quote, QuotePort, QuoteQuery};

pub struct QuoteRepoFile {
    path: PathBuf
}

impl QuoteRepoFile {
    pub fn new(path: PathBuf) -> Self {
        QuoteRepoFile { path }
    }
}

#[async_trait]
impl QuotePort for QuoteRepoFile {
    async fn find_by_id(&self, query: QuoteQuery) -> Result<Quote, AppError> {
        todo!()
    }

    async fn random_find_by_content_key(&self, query: QuoteQuery) -> Result<Quote, AppError> {
        todo!()
    }
}