use super::create_quote::{detect_content_type, UploadKind};
use crate::application::quote::{QuotePort, QuoteQuery, QuoteUpdate};
use crate::application::service::storage::{
    DeleteManyService, UploadManyWithRollbackService, UploadObjectItem,
};
use crate::application::storage::StoragePayload;
use crate::application::ApplicationError;
use crate::domain::entity::{MultiLangText, Quote};
use crate::domain::value::{Lang, ObjectKey};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Default)]
pub struct QuoteUpdateDraft {
    pub id: i64,
    pub inline: Option<MultiLangText>,
    pub external: Option<HashMap<Lang, StoragePayload>>,
    pub markdown: Option<HashMap<Lang, StoragePayload>>,
    pub image: Option<Vec<StoragePayload>>,
    pub remark: Option<Option<String>>,
}

/// 更新 Quote 的服务。
///
/// 流程：
/// - 按 id 读取旧数据。
/// - 上传新对象（仅针对 external/markdown/image 传入的字段）。
/// - 调用 `QuotePort::update`。
/// - 失败时回滚新对象；成功后清理被替换的旧对象。
pub struct UpdateQuoteService<'a> {
    quote_port: &'a (dyn QuotePort + Send + Sync),
    upload_service: UploadManyWithRollbackService<'a>,
    delete_service: DeleteManyService<'a>,
}

impl<'a> UpdateQuoteService<'a> {
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

    pub async fn execute(&self, draft: QuoteUpdateDraft) -> Result<Quote, ApplicationError> {
        if draft.id <= 0 {
            return Err(ApplicationError::InvalidInput(
                "id must be greater than 0".to_string(),
            ));
        }
        if draft.inline.is_none()
            && draft.external.is_none()
            && draft.markdown.is_none()
            && draft.image.is_none()
            && draft.remark.is_none()
        {
            return Err(ApplicationError::InvalidInput(
                "no fields to update".to_string(),
            ));
        }
        validate_inline_update_draft(&draft)?;

        let previous = self
            .quote_port
            .get(QuoteQuery::builder().id(draft.id).build())
            .await?;

        if !has_content_after_update(&previous, &draft) {
            return Err(ApplicationError::InvalidInput(
                "quote content is missing after update".to_string(),
            ));
        }

        let mut plan = UpdateUploadPlan::from_update_draft(&draft)?;
        let items = std::mem::take(&mut plan.items);
        let uploaded = self.upload_service.execute(items).await?;
        let update = plan.to_update(&draft, &previous, &uploaded)?;

        let updated = match self.quote_port.update(update).await {
            Ok(quote) => quote,
            Err(err) => {
                let rollback_err = self.delete_service.execute(&uploaded).await.err();
                if let Some(cleanup_err) = rollback_err {
                    return Err(ApplicationError::Dependency(format!(
                        "update failed: {err}; cleanup failed: {cleanup_err}"
                    )));
                }
                return Err(err);
            }
        };

        let keys_to_cleanup = plan.keys_to_cleanup(&previous, &updated);
        if keys_to_cleanup.is_empty() {
            return Ok(updated);
        }

        if let Err(cleanup_err) = self.delete_service.execute(&keys_to_cleanup).await {
            return Err(ApplicationError::Dependency(format!(
                "quote updated, but old objects cleanup failed: {cleanup_err}"
            )));
        }

        Ok(updated)
    }
}

fn has_content_after_update(previous: &Quote, draft: &QuoteUpdateDraft) -> bool {
    let inline_non_empty = draft
        .inline
        .as_ref()
        .map(|v| v.values().any(|text| !text.trim().is_empty()))
        .unwrap_or_else(|| previous.inline().values().any(|text| !text.trim().is_empty()));

    let external_non_empty = draft
        .external
        .as_ref()
        .map(|v| !v.is_empty())
        .unwrap_or(!previous.external().is_empty());

    let markdown_non_empty = draft
        .markdown
        .as_ref()
        .map(|v| !v.is_empty())
        .unwrap_or(!previous.markdown().is_empty());

    let image_non_empty = draft
        .image
        .as_ref()
        .map(|v| !v.is_empty())
        .unwrap_or(!previous.image().is_empty());

    inline_non_empty || external_non_empty || markdown_non_empty || image_non_empty
}

fn validate_inline_update_draft(draft: &QuoteUpdateDraft) -> Result<(), ApplicationError> {
    if let Some(inline) = &draft.inline {
        for text in inline.values() {
            if text.trim().is_empty() {
                return Err(ApplicationError::InvalidInput(
                    "inline text must not be blank".to_string(),
                ));
            }
        }
    }
    Ok(())
}

#[derive(Debug)]
struct UpdateUploadPlan {
    items: Vec<UploadObjectItem>,
    replace_external: bool,
    replace_markdown: bool,
    replace_image: bool,
    external_langs: Vec<Lang>,
    markdown_langs: Vec<Lang>,
    image_count: usize,
}

impl UpdateUploadPlan {
    fn from_update_draft(draft: &QuoteUpdateDraft) -> Result<Self, ApplicationError> {
        let mut items = Vec::new();
        let mut external_langs = Vec::new();
        let mut markdown_langs = Vec::new();
        let mut image_count = 0usize;

        let replace_external = draft.external.is_some();
        if let Some(external) = &draft.external {
            let mut pairs: Vec<_> = external.iter().collect();
            pairs.sort_by(|a, b| a.0.as_str().cmp(b.0.as_str()));

            external_langs.reserve(pairs.len());
            for (lang, payload) in pairs {
                items.push(UploadObjectItem {
                    path: format!("text/{}", lang.as_str()),
                    payload: payload.clone(),
                    content_type: detect_content_type(payload, UploadKind::Text)?,
                });
                external_langs.push((*lang).clone());
            }
        }

        let replace_markdown = draft.markdown.is_some();
        if let Some(markdown) = &draft.markdown {
            let mut pairs: Vec<_> = markdown.iter().collect();
            pairs.sort_by(|a, b| a.0.as_str().cmp(b.0.as_str()));

            markdown_langs.reserve(pairs.len());
            for (lang, payload) in pairs {
                items.push(UploadObjectItem {
                    path: format!("markdown/{}", lang.as_str()),
                    payload: payload.clone(),
                    content_type: detect_content_type(payload, UploadKind::Markdown)?,
                });
                markdown_langs.push((*lang).clone());
            }
        }

        let replace_image = draft.image.is_some();
        if let Some(image) = &draft.image {
            image_count = image.len();
            for payload in image {
                let content_type = detect_content_type(payload, UploadKind::Image)?;
                items.push(UploadObjectItem {
                    path: "image".to_string(),
                    payload: payload.clone(),
                    content_type,
                });
            }
        }

        Ok(Self {
            items,
            replace_external,
            replace_markdown,
            replace_image,
            external_langs,
            markdown_langs,
            image_count,
        })
    }

    fn to_update(
        &self,
        draft: &QuoteUpdateDraft,
        previous: &Quote,
        uploaded: &[ObjectKey],
    ) -> Result<QuoteUpdate, ApplicationError> {
        let expected =
            self.external_langs.len() + self.markdown_langs.len() + self.image_count;
        if uploaded.len() != expected {
            return Err(ApplicationError::Dependency(format!(
                "uploaded key count mismatch: expected {expected}, got {}",
                uploaded.len()
            )));
        }

        let mut idx = 0usize;

        let external = if self.replace_external {
            // patch merge: 未传语言保持原值，传入语言覆盖为新上传 key。
            let mut map = previous.external().clone();
            for lang in &self.external_langs {
                map.insert(lang.clone(), uploaded[idx].clone());
                idx += 1;
            }
            Some(map)
        } else {
            None
        };

        let markdown = if self.replace_markdown {
            // patch merge: 未传语言保持原值，传入语言覆盖为新上传 key。
            let mut map = previous.markdown().clone();
            for lang in &self.markdown_langs {
                map.insert(lang.clone(), uploaded[idx].clone());
                idx += 1;
            }
            Some(map)
        } else {
            None
        };

        let image = if self.replace_image {
            // patch merge: image 使用追加语义。
            let mut merged = previous.image().to_vec();
            merged.extend_from_slice(&uploaded[idx..]);
            Some(merged)
        } else {
            None
        };

        let inline = if let Some(inline_patch) = &draft.inline {
            // patch merge: 未传语言保持原值，传入语言覆盖文本。
            let mut merged = previous.inline().clone();
            for (lang, text) in inline_patch {
                merged.insert(lang.clone(), text.clone());
            }
            Some(merged)
        } else {
            None
        };

        Ok(QuoteUpdate {
            id: draft.id,
            inline,
            external,
            markdown,
            image,
            remark: draft.remark.clone(),
        })
    }

    fn keys_to_cleanup(&self, previous: &Quote, updated: &Quote) -> Vec<ObjectKey> {
        let mut keys = Vec::new();
        let mut dedup = HashSet::new();
        let retained: HashSet<&str> = updated
            .external()
            .values()
            .map(|k| k.as_str())
            .chain(updated.markdown().values().map(|k| k.as_str()))
            .chain(updated.image().iter().map(|k| k.as_str()))
            .collect();

        if self.replace_external {
            for key in previous.external().values() {
                if retained.contains(key.as_str()) {
                    continue;
                }
                if dedup.insert(key.as_str().to_string()) {
                    keys.push(key.clone());
                }
            }
        }
        if self.replace_markdown {
            for key in previous.markdown().values() {
                if retained.contains(key.as_str()) {
                    continue;
                }
                if dedup.insert(key.as_str().to_string()) {
                    keys.push(key.clone());
                }
            }
        }
        if self.replace_image {
            for key in previous.image() {
                if retained.contains(key.as_str()) {
                    continue;
                }
                if dedup.insert(key.as_str().to_string()) {
                    keys.push(key.clone());
                }
            }
        }

        keys
    }
}
