use crate::app::AppError;
use crate::quote::{Quote, QuotePort, QuoteQuery};

const DEFAULT_PAGE_SIZE: i64 = 10;
const MAX_PAGE_SIZE: i64 = 100;

pub struct ListQuotes<'a> {
    port: &'a (dyn QuotePort + Send + Sync),
}

impl<'a> ListQuotes<'a> {
    pub fn new(port: &'a (dyn QuotePort + Send + Sync)) -> Self {
        Self { port }
    }

    pub async fn execute(&self, query: QuoteQuery) -> Result<Vec<Quote>, AppError> {
        let limit = match query.limit() {
            Some(l) if l > 0 => l.min(MAX_PAGE_SIZE),
            _ => DEFAULT_PAGE_SIZE,
        };
        let offset = query.offset().unwrap_or(0);
        let new_query = QuoteQuery::builder()
            .with_filter(query.filter().cloned())
            .with_active(query.active())
            .limit(limit)
            .offset(offset)
            .build();
        self.port.list(new_query).await
    }
}
