use async_trait::async_trait;
use crate::app::app_error::AppError;
use crate::quote::{Quote, QuoteQuery};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait QuotePort {
    async fn find_by_id(&self, query: QuoteQuery) -> Result<Quote, AppError>;

    async fn random_find_by_content_key(&self, query: QuoteQuery) -> Result<Quote, AppError>;
}