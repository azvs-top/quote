use crate::application::ApplicationError;
use crate::application::quote::{QuotePort, QuoteQuery};

/// 统计 Quote 条数的服务。
pub struct CountQuoteService<'a> {
    port: &'a (dyn QuotePort + Send + Sync),
}

impl<'a> CountQuoteService<'a> {
    pub fn new(port: &'a (dyn QuotePort + Send + Sync)) -> Self {
        Self { port }
    }

    pub async fn execute(&self, query: QuoteQuery) -> Result<i64, ApplicationError> {
        self.port.count(query).await
    }
}
