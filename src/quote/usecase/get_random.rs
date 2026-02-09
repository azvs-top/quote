use crate::app::AppError;
use crate::quote::{Quote, QuotePort, QuoteQuery};

pub struct GetQuoteRandom<'a> {
    port: &'a dyn QuotePort,
}

impl<'a> GetQuoteRandom<'a> {
    pub fn new(port: &'a dyn QuotePort) -> Self {
        Self { port }
    }

    pub async fn execute(&self, query: QuoteQuery) -> Result<Quote, AppError> {
        let new_query = QuoteQuery::builder()
            .with_filter(query.filter().cloned())
            .with_active(query.active())
            .build();
        self.port.get(new_query)
            .await
            .map_err(|_| AppError::QuoteNotFound)

    }
}