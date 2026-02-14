use crate::app::AppError;
use crate::quote::{Quote, QuoteFilePayload, QuotePort, QuoteQuery};
use serde_json::Value;

pub struct AddAudioById<'a> {
    port: &'a (dyn QuotePort + Send + Sync),
}

impl<'a> AddAudioById<'a> {
    pub fn new(port: &'a (dyn QuotePort + Send + Sync)) -> Self {
        Self { port }
    }

    pub async fn execute(
        &self,
        id: i64,
        group: impl Into<String>,
        payload: QuoteFilePayload,
    ) -> Result<Quote, AppError> {
        let group = group.into();
        let content_type = guess_mime(&payload);
        let key = self
            .port
            .upload_object("audio", payload, content_type)
            .await?;

        let mut quote = self
            .port
            .get(QuoteQuery::builder().id(id).build())
            .await?;

        let root = quote
            .content
            .as_object_mut()
            .ok_or(AppError::QuoteInvalidContent)?;
        let audio = root
            .entry("audio")
            .or_insert_with(|| Value::Object(serde_json::Map::new()))
            .as_object_mut()
            .ok_or(AppError::QuoteInvalidContent)?;
        let arr = audio
            .entry(group)
            .or_insert_with(|| Value::Array(vec![Value::String(key.clone())]));
        if !arr.is_array() {
            return Err(AppError::QuoteInvalidContent);
        }
        *arr = Value::Array(vec![Value::String(key)]);

        self.port.update_content(id, quote.content).await
    }
}

fn guess_mime(file: &QuoteFilePayload) -> &'static str {
    infer::get(&file.bytes)
        .map(|kind| kind.mime_type())
        .unwrap_or("application/octet-stream")
}
