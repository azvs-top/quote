use crate::app::AppError;
use crate::quote::{Quote, QuoteAdd, QuoteAddDraft, QuoteFilePayload, QuotePort};
use serde_json::{Map, Value};

pub struct CreateQuote<'a> {
    port: &'a (dyn QuotePort + Send + Sync),
}

impl<'a> CreateQuote<'a> {
    pub fn new(port: &'a (dyn QuotePort + Send + Sync)) -> Self {
        Self { port }
    }

    pub async fn execute(&self, draft: QuoteAddDraft) -> Result<Quote, AppError> {
        let content = self.build_content(&draft).await?;
        self.port
            .add(QuoteAdd {
                content: Value::Object(content),
                active: draft.active,
                remark: draft.remark,
            })
            .await
    }

    async fn build_content(&self, draft: &QuoteAddDraft) -> Result<Map<String, Value>, AppError> {
        let mut content = Map::new();

        if !draft.inline.is_empty() {
            let mut inline = Map::new();
            for (lang, txt) in &draft.inline {
                inline.insert(lang.clone(), Value::String(txt.clone()));
            }
            content.insert("inline".to_string(), Value::Object(inline));
        }

        if !draft.external.is_empty() {
            let mut external = Map::new();
            for (lang, file) in &draft.external {
                let key = self
                    .port
                    .upload_object(
                        &format!("text/{}", lang),
                        file.clone(),
                        "text/plain; charset=utf-8",
                    )
                    .await?;
                external.insert(lang.clone(), Value::String(key));
            }
            content.insert("external".to_string(), Value::Object(external));
        }

        if !draft.markdown.is_empty() {
            let mut markdown = Map::new();
            for (lang, file) in &draft.markdown {
                let key = self
                    .port
                    .upload_object(
                        &format!("markdown/{}", lang),
                        file.clone(),
                        "text/markdown; charset=utf-8",
                    )
                    .await?;
                markdown.insert(lang.clone(), Value::String(key));
            }
            content.insert("markdown".to_string(), Value::Object(markdown));
        }

        if !draft.image.is_empty() {
            let mut image = Vec::new();
            for file in &draft.image {
                let key = self
                    .port
                    .upload_object("image", file.clone(), guess_mime(file))
                    .await?;
                image.push(Value::String(key));
            }
            content.insert("image".to_string(), Value::Array(image));
        }

        if !draft.audio.is_empty() {
            let mut audio = Map::new();
            for (group, files) in &draft.audio {
                let mut keys = Vec::new();
                for file in files {
                    let key = self
                        .port
                        .upload_object("audio", file.clone(), guess_mime(file))
                        .await?;
                    keys.push(Value::String(key));
                }
                audio.insert(group.clone(), Value::Array(keys));
            }
            content.insert("audio".to_string(), Value::Object(audio));
        }

        if content.is_empty() {
            return Err(AppError::QuoteInvalidContent);
        }

        Ok(content)
    }
}

fn guess_mime(file: &QuoteFilePayload) -> &'static str {
    infer::get(&file.bytes)
        .map(|kind| kind.mime_type())
        .unwrap_or("application/octet-stream")
}
