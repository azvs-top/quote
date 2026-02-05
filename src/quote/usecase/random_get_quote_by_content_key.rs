use crate::app::AppError;
use crate::PgJson;
use crate::quote::{Quote, QuotePort, QuoteQuery};

pub struct RandomGetQuoteByContentKey<'a> {
    port: &'a dyn QuotePort,
}

impl<'a> RandomGetQuoteByContentKey<'a> {
    pub fn new(port: &'a dyn QuotePort) -> Self {
        Self { port }
    }

    pub async fn execute(&self) -> Result<Quote, AppError> {
        let query = QuoteQuery::builder()
            .build();
        self.port
            .random_find_by_content_key(query)
            .await
            .map_err(|_| AppError::QuoteNotFound)
    }
}