use crate::app::AppError;
use crate::quote::{Quote, QuotePort, QuoteQuery, QuoteQueryFilter};

pub struct RandomGetQuoteByContentKey<'a> {
    port: &'a dyn QuotePort,
}

impl<'a> RandomGetQuoteByContentKey<'a> {
    pub fn new(port: &'a dyn QuotePort) -> Self {
        Self { port }
    }

    pub async fn execute(&self, langs: Vec<String>) -> Result<Quote, AppError> {
        let filter = QuoteQueryFilter::AllLangs(langs);

        let query = QuoteQuery::builder()
            .filter(filter)
            .build();
        self.port
            .random_get_by_content_key(query)
            .await
            .map_err(|_| AppError::QuoteNotFound)
    }
}