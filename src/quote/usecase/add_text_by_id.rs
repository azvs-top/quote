use crate::app::AppError;
use crate::quote::{Quote, QuotePort, QuoteQuery};
use serde_json::Value;

pub struct AddTextById<'a> {
    port: &'a (dyn QuotePort + Send + Sync),
}

impl<'a> AddTextById<'a> {
    pub fn new(port: &'a (dyn QuotePort + Send + Sync)) -> Self {
        Self { port }
    }

    pub async fn execute(
        &self,
        id: i64,
        lang: impl Into<String>,
        text: impl Into<String>,
    ) -> Result<Quote, AppError> {
        let mut quote = self
            .port
            .get(QuoteQuery::builder().id(id).build())
            .await?;

        let root = quote
            .content
            .as_object_mut()
            .ok_or(AppError::QuoteInvalidContent)?;
        let inline = root
            .entry("inline")
            .or_insert_with(|| Value::Object(serde_json::Map::new()))
            .as_object_mut()
            .ok_or(AppError::QuoteInvalidContent)?;

        inline.insert(lang.into(), Value::String(text.into()));

        self.port.update_content(id, quote.content).await
    }
}
