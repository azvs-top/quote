use crate::application::ApplicationError;
use crate::application::quote::{QuotePort, QuoteQuery};
use crate::application::service::storage::DeleteManyService;
use crate::application::storage::StoragePort;
use crate::domain::quote::{Quote, QuotePatch};
use crate::domain::value::{Lang, ObjectKey};
use std::collections::HashSet;

#[derive(Debug, Clone, Default)]
pub struct PartialDeleteQuoteDraft {
    pub id: i64,
    pub inline_langs: Vec<Lang>,
    pub clear_inline: bool,
    pub external_langs: Vec<Lang>,
    pub clear_external: bool,
    pub markdown_langs: Vec<Lang>,
    pub clear_markdown: bool,
    pub image_keys: Vec<ObjectKey>,
    pub image_indexes: Vec<usize>,
    pub clear_image: bool,
}

pub struct PartialDeleteQuoteService<'a> {
    quote_port: &'a (dyn QuotePort + Send + Sync),
    delete_many_service: DeleteManyService<'a>,
}

impl<'a> PartialDeleteQuoteService<'a> {
    pub fn new(
        quote_port: &'a (dyn QuotePort + Send + Sync),
        storage_port: &'a (dyn StoragePort + Send + Sync),
    ) -> Self {
        Self {
            quote_port,
            delete_many_service: DeleteManyService::new(storage_port),
        }
    }

    pub async fn execute(&self, draft: PartialDeleteQuoteDraft) -> Result<Quote, ApplicationError> {
        if draft.id <= 0 {
            return Err(ApplicationError::InvalidInput(
                "id must be greater than 0".to_string(),
            ));
        }
        if !has_any_delete_action(&draft) {
            return Err(ApplicationError::InvalidInput(
                "no partial delete actions provided".to_string(),
            ));
        }

        let current = self
            .quote_port
            .get(QuoteQuery::builder().id(draft.id).build())
            .await?;

        let mut removed_object_keys = Vec::new();

        if draft.clear_external {
            removed_object_keys.extend(current.external().values().cloned());
        } else {
            for lang in &draft.external_langs {
                if let Some(key) = current.external().get(lang) {
                    removed_object_keys.push(key.clone());
                }
            }
        }

        if draft.clear_markdown {
            removed_object_keys.extend(current.markdown().values().cloned());
        } else {
            for lang in &draft.markdown_langs {
                if let Some(key) = current.markdown().get(lang) {
                    removed_object_keys.push(key.clone());
                }
            }
        }

        let remove_image = if draft.clear_image {
            current.image().to_vec()
        } else {
            let mut resolved = draft.image_keys.clone();
            for index in &draft.image_indexes {
                let Some(key) = current.image().get(*index) else {
                    return Err(ApplicationError::InvalidInput(format!(
                        "image index out of range: {index}"
                    )));
                };
                resolved.push(key.clone());
            }
            resolved
        };
        removed_object_keys.extend(remove_image.iter().cloned());

        let patch = QuotePatch::new(
            None,
            draft.clear_inline,
            draft.inline_langs.clone(),
            None,
            draft.clear_external,
            draft.external_langs.clone(),
            None,
            draft.clear_markdown,
            draft.markdown_langs.clone(),
            None,
            draft.clear_image,
            remove_image,
            None,
        )
            .map_err(ApplicationError::from)?;
        let updated = self.quote_port.update(draft.id, patch).await?;

        if removed_object_keys.is_empty() {
            return Ok(updated);
        }

        let keys_for_cleanup = dedup_object_keys(removed_object_keys);
        if let Err(cleanup_err) = self.delete_many_service.execute(&keys_for_cleanup).await {
            return Err(ApplicationError::Dependency(format!(
                "quote updated, but removed object cleanup failed: {cleanup_err}"
            )));
        }

        Ok(updated)
    }
}

fn has_any_delete_action(draft: &PartialDeleteQuoteDraft) -> bool {
    draft.clear_inline
        || !draft.inline_langs.is_empty()
        || draft.clear_external
        || !draft.external_langs.is_empty()
        || draft.clear_markdown
        || !draft.markdown_langs.is_empty()
        || draft.clear_image
        || !draft.image_keys.is_empty()
        || !draft.image_indexes.is_empty()
}

fn dedup_object_keys(keys: Vec<ObjectKey>) -> Vec<ObjectKey> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for key in keys {
        if seen.insert(key.as_str().to_string()) {
            out.push(key);
        }
    }
    out
}
