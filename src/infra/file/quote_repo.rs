use std::path::PathBuf;
use async_trait::async_trait;
use crate::app::app_error::AppError;
use crate::quote::{Quote, QuoteAdd, QuoteFilePayload, QuotePort, QuoteQuery, QuoteQueryFilter};
use serde_json::Value;

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
    async fn upload_object(
        &self,
        path: &str,
        payload: QuoteFilePayload,
        content_type: &str,
    ) -> Result<String, AppError> {
        todo!()
    }

    async fn add(&self, add: QuoteAdd) -> Result<Quote, AppError> {
        todo!()
    }

    async fn update_content(&self, id: i64, content: Value) -> Result<Quote, AppError> {
        todo!()
    }

    async fn get(&self, query: QuoteQuery) -> Result<Quote, AppError> {
        todo!()
    }

    async fn list(&self, query: QuoteQuery) -> Result<Vec<Quote>, AppError> {
        todo!()
    }
}
