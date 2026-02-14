use crate::app::AppError;
use crate::quote::{Quote, QuoteFilePayload, QuotePort, QuoteQuery};
use serde_json::Value;

pub struct AddImageById<'a> {
    port: &'a (dyn QuotePort + Send + Sync),
}

impl<'a> AddImageById<'a> {
    pub fn new(port: &'a (dyn QuotePort + Send + Sync)) -> Self {
        Self { port }
    }

    pub async fn execute(
        &self,
        id: i64,
        payload: QuoteFilePayload,
    ) -> Result<Quote, AppError> {
        let content_type = guess_mime(&payload);
        let key = self
            .port
            .upload_object("image", payload, content_type)
            .await?;

        let mut quote = self
            .port
            .get(QuoteQuery::builder().id(id).build())
            .await?;

        let root = quote
            .content
            .as_object_mut()
            .ok_or(AppError::QuoteInvalidContent)?;
        let image = root
            .entry("image")
            .or_insert_with(|| Value::Array(Vec::new()));

        // Backward compatibility:
        // If stored as legacy object like {"cover": [...]}, flatten into array.
        if image.is_object() {
            let mut merged = Vec::new();
            let obj = image.as_object().ok_or(AppError::QuoteInvalidContent)?;
            for value in obj.values() {
                let arr = value.as_array().ok_or(AppError::QuoteInvalidContent)?;
                for item in arr {
                    let s = item.as_str().ok_or(AppError::QuoteInvalidContent)?;
                    merged.push(Value::String(s.to_string()));
                }
            }
            *image = Value::Array(merged);
        }

        let arr = image
            .as_array_mut()
            .ok_or(AppError::QuoteInvalidContent)?;
        arr.push(Value::String(key));

        self.port.update_content(id, quote.content).await
    }
}

fn guess_mime(file: &QuoteFilePayload) -> &'static str {
    infer::get(&file.bytes)
        .map(|kind| kind.mime_type())
        .unwrap_or("application/octet-stream")
}
