use crate::app::AppError;
use crate::quote::{Quote, QuoteFilePayload, QuotePort, QuoteQuery};
use serde_json::Value;

pub struct AddTextFileById<'a> {
    port: &'a (dyn QuotePort + Send + Sync),
}

impl<'a> AddTextFileById<'a> {
    pub fn new(port: &'a (dyn QuotePort + Send + Sync)) -> Self {
        Self { port }
    }

    pub async fn execute(
        &self,
        id: i64,
        lang: impl Into<String>,
        payload: QuoteFilePayload,
    ) -> Result<Quote, AppError> {
        let lang = lang.into();
        let key = self
            .port
            .upload_object(
                &format!("text/{}", lang),
                payload,
                "text/plain; charset=utf-8",
            )
            .await?;

        let mut quote = self
            .port
            .get(QuoteQuery::builder().id(id).build())
            .await?;

        let root = quote
            .content
            .as_object_mut()
            .ok_or(AppError::QuoteInvalidContent)?;
        let external = root
            .entry("external")
            .or_insert_with(|| Value::Object(serde_json::Map::new()))
            .as_object_mut()
            .ok_or(AppError::QuoteInvalidContent)?;

        external.insert(lang, Value::String(key));

        self.port.update_content(id, quote.content).await
    }
}
