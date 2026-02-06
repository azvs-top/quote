use crate::app::app_error::AppError;
use crate::quote::{Quote, QuotePort, QuoteQuery};

pub struct GetQuoteById<'a> {
    port: &'a dyn QuotePort,
}
impl<'a> GetQuoteById<'a> {
    pub fn new(port: &'a dyn QuotePort) -> Self {
        Self { port }
    }

    pub async fn execute(&self, id: i64) -> Result<Quote, AppError> {
        let query = QuoteQuery::builder()
            .id(id)
            .build();
        self.port
            .get(query)
            .await
            .map_err(|_| AppError::QuoteNotFound)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use crate::app::app_error::AppError;
    use crate::quote::{GetQuoteById, MockQuotePort, Quote};

    #[tokio::test]
    async fn get_quote_by_id_success() {
        let mut mock = MockQuotePort::new();
        mock.expect_get()
            .returning(|_| {
                Ok(Quote {
                    id: 1,
                    content: json!({"test": "hello"}),
                    active: true,
                    remark: None,
                })
            });

        let quote = GetQuoteById::new(&mock).execute(1).await.unwrap();
        println!("{:?}", quote);
        assert_eq!(quote.id, 1);
    }

    #[tokio::test]
    async fn get_quote_by_id_failure() {
        let mut mock = MockQuotePort::new();
        mock.expect_get()
            .returning(|_| Err(AppError::QuoteNotFound));

        let err = GetQuoteById::new(&mock).execute(1).await.unwrap_err();
        println!("{:?}: {}", err, err);
        assert!(matches!(err, AppError::QuoteNotFound));
    }
}