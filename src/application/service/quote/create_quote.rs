use crate::application::ApplicationError;
use crate::application::quote::QuotePort;
use crate::application::service::storage::{
    DeleteManyService, UploadManyWithRollbackService, UploadObjectItem,
};
use crate::application::storage::StoragePayload;
use crate::domain::quote::{MultiLangObject, MultiLangText, Quote, QuoteDraft};
use crate::domain::value::{Lang, ObjectKey};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct QuoteCreateDraft {
    pub inline: MultiLangText,
    pub external: HashMap<Lang, StoragePayload>,
    pub markdown: HashMap<Lang, StoragePayload>,
    pub image: Vec<StoragePayload>,
    pub remark: Option<String>,
}

/// 创建 Quote 的服务。
///
/// 流程：
/// - 先上传草稿中的对象（external/markdown/image）。
/// - 再调用 `QuotePort::create` 落库。
/// - 若落库失败，补偿删除已上传对象。
pub struct CreateQuoteService<'a> {
    quote_port: &'a (dyn QuotePort + Send + Sync),
    upload_service: UploadManyWithRollbackService<'a>,
    delete_service: DeleteManyService<'a>,
}

impl<'a> CreateQuoteService<'a> {
    pub fn new(
        quote_port: &'a (dyn QuotePort + Send + Sync),
        storage_port: &'a (dyn crate::application::storage::StoragePort + Send + Sync),
    ) -> Self {
        Self {
            quote_port,
            upload_service: UploadManyWithRollbackService::new(storage_port),
            delete_service: DeleteManyService::new(storage_port),
        }
    }

    pub async fn execute(&self, draft: QuoteCreateDraft) -> Result<Quote, ApplicationError> {
        if draft.inline.is_empty()
            && draft.external.is_empty()
            && draft.markdown.is_empty()
            && draft.image.is_empty()
        {
            return Err(ApplicationError::InvalidInput(
                "quote content is missing".to_string(),
            ));
        }

        let mut plan = UploadPlan::from_create_draft(draft)?;
        let items = std::mem::take(&mut plan.items);
        let uploaded = self.upload_service.execute(items).await?;
        let create = plan.to_create(&uploaded)?;

        match self.quote_port.create(create).await {
            Ok(quote) => Ok(quote),
            Err(err) => {
                let rollback_err = self.delete_service.execute(&uploaded).await.err();

                if let Some(cleanup_err) = rollback_err {
                    return Err(ApplicationError::Dependency(format!(
                        "create failed: {err}; cleanup failed: {cleanup_err}"
                    )));
                }
                Err(err)
            }
        }
    }
}

#[derive(Debug)]
struct UploadPlan {
    items: Vec<UploadObjectItem>,
    inline: MultiLangText,
    remark: Option<String>,
    external_langs: Vec<Lang>,
    markdown_langs: Vec<Lang>,
    image_count: usize,
}

impl UploadPlan {
    fn from_create_draft(draft: QuoteCreateDraft) -> Result<Self, ApplicationError> {
        let mut items =
            Vec::with_capacity(draft.external.len() + draft.markdown.len() + draft.image.len());

        let mut external_pairs: Vec<_> = draft.external.iter().collect();
        external_pairs.sort_by(|a, b| a.0.as_str().cmp(b.0.as_str()));

        let mut markdown_pairs: Vec<_> = draft.markdown.iter().collect();
        markdown_pairs.sort_by(|a, b| a.0.as_str().cmp(b.0.as_str()));

        let mut external_langs = Vec::with_capacity(external_pairs.len());
        for (lang, payload) in external_pairs {
            let content_type = detect_content_type(payload, UploadKind::Text)?;
            items.push(UploadObjectItem {
                path: format!("text/{}", lang.as_str()),
                payload: payload.clone(),
                content_type,
            });
            external_langs.push((*lang).clone());
        }

        let mut markdown_langs = Vec::with_capacity(markdown_pairs.len());
        for (lang, payload) in markdown_pairs {
            let content_type = detect_content_type(payload, UploadKind::Markdown)?;
            items.push(UploadObjectItem {
                path: format!("markdown/{}", lang.as_str()),
                payload: payload.clone(),
                content_type,
            });
            markdown_langs.push((*lang).clone());
        }

        for payload in &draft.image {
            let content_type = detect_content_type(payload, UploadKind::Image)?;
            items.push(UploadObjectItem {
                path: "image".to_string(),
                payload: payload.clone(),
                content_type,
            });
        }

        Ok(Self {
            items,
            inline: draft.inline,
            remark: draft.remark,
            external_langs,
            markdown_langs,
            image_count: draft.image.len(),
        })
    }

    fn to_create(&self, uploaded: &[ObjectKey]) -> Result<QuoteDraft, ApplicationError> {
        let expected = self.external_langs.len() + self.markdown_langs.len() + self.image_count;
        if uploaded.len() != expected {
            return Err(ApplicationError::Dependency(format!(
                "uploaded key count mismatch: expected {expected}, got {}",
                uploaded.len()
            )));
        }

        let mut idx = 0usize;
        let mut external = MultiLangObject::new();
        for lang in &self.external_langs {
            external.insert(lang.clone(), uploaded[idx].clone());
            idx += 1;
        }

        let mut markdown = MultiLangObject::new();
        for lang in &self.markdown_langs {
            markdown.insert(lang.clone(), uploaded[idx].clone());
            idx += 1;
        }

        let image = uploaded[idx..].to_vec();

        QuoteDraft::new(
            self.inline.clone(),
            external,
            markdown,
            image,
            self.remark.clone(),
        )
        .map_err(ApplicationError::from)
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) enum UploadKind {
    Text,
    Markdown,
    Image,
}

pub(super) fn detect_content_type(
    payload: &StoragePayload,
    kind: UploadKind,
) -> Result<String, ApplicationError> {
    let mut candidate = if let Some(kind_meta) = infer::get(&payload.bytes) {
        kind_meta.mime_type().to_string()
    } else if let Some(filename) = payload.filename.as_deref() {
        mime_from_filename(filename)
    } else {
        default_mime(kind).to_string()
    };

    // 若 infer 给出的是 text/plain，但文件扩展名更明确（如 .md），优先使用扩展名。
    if let Some(filename) = payload.filename.as_deref() {
        let by_name = mime_from_filename(filename);
        if candidate == "text/plain" && by_name != "text/plain; charset=utf-8" {
            candidate = by_name;
        }
    }

    match kind {
        UploadKind::Text => {
            if !base_mime(&candidate).starts_with("text/") {
                return Err(ApplicationError::InvalidInput(format!(
                    "external text must be text/*, got {candidate}"
                )));
            }
            Ok(with_utf8(candidate))
        }
        UploadKind::Markdown => {
            let base = base_mime(&candidate);
            if base != "text/markdown" && base != "text/plain" {
                return Err(ApplicationError::InvalidInput(format!(
                    "markdown must be text/markdown or text/plain, got {candidate}"
                )));
            }
            Ok("text/markdown; charset=utf-8".to_string())
        }
        UploadKind::Image => {
            if !base_mime(&candidate).starts_with("image/") {
                return Err(ApplicationError::InvalidInput(format!(
                    "image payload must be image/*, got {candidate}"
                )));
            }
            Ok(base_mime(&candidate).to_string())
        }
    }
}

fn mime_from_filename(filename: &str) -> String {
    let lower = filename.to_ascii_lowercase();
    if lower.ends_with(".md") || lower.ends_with(".markdown") {
        return "text/markdown; charset=utf-8".to_string();
    }
    if lower.ends_with(".txt") {
        return "text/plain; charset=utf-8".to_string();
    }
    if lower.ends_with(".png") {
        return "image/png".to_string();
    }
    if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        return "image/jpeg".to_string();
    }
    if lower.ends_with(".gif") {
        return "image/gif".to_string();
    }
    if lower.ends_with(".webp") {
        return "image/webp".to_string();
    }
    if lower.ends_with(".svg") {
        return "image/svg+xml".to_string();
    }
    "text/plain; charset=utf-8".to_string()
}

fn default_mime(kind: UploadKind) -> &'static str {
    match kind {
        UploadKind::Text => "text/plain; charset=utf-8",
        UploadKind::Markdown => "text/markdown; charset=utf-8",
        UploadKind::Image => "application/octet-stream",
    }
}

fn base_mime(mime: &str) -> &str {
    mime.split(';').next().unwrap_or(mime).trim()
}

fn with_utf8(mime: String) -> String {
    if mime.contains("charset=") {
        return mime;
    }
    if base_mime(&mime).starts_with("text/") {
        return format!("{}; charset=utf-8", base_mime(&mime));
    }
    mime
}
