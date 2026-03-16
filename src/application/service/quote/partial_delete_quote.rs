use crate::application::ApplicationError;
use crate::application::quote::{QuotePort, QuoteQuery, QuoteUpdate};
use crate::application::service::storage::DeleteManyService;
use crate::application::storage::StoragePort;
use crate::domain::entity::Quote;
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
        let mut update = QuoteUpdate {
            id: draft.id,
            ..Default::default()
        };

        if draft.clear_inline || !draft.inline_langs.is_empty() {
            let mut inline = current.inline().clone();
            if draft.clear_inline {
                inline.clear();
            } else {
                for lang in &draft.inline_langs {
                    inline.remove(lang);
                }
            }
            update.inline = Some(inline);
        }

        if draft.clear_external || !draft.external_langs.is_empty() {
            let mut external = current.external().clone();
            if draft.clear_external {
                removed_object_keys.extend(external.values().cloned());
                external.clear();
            } else {
                for lang in &draft.external_langs {
                    if let Some(key) = external.remove(lang) {
                        removed_object_keys.push(key);
                    }
                }
            }
            update.external = Some(external);
        }

        if draft.clear_markdown || !draft.markdown_langs.is_empty() {
            let mut markdown = current.markdown().clone();
            if draft.clear_markdown {
                removed_object_keys.extend(markdown.values().cloned());
                markdown.clear();
            } else {
                for lang in &draft.markdown_langs {
                    if let Some(key) = markdown.remove(lang) {
                        removed_object_keys.push(key);
                    }
                }
            }
            update.markdown = Some(markdown);
        }

        if draft.clear_image || !draft.image_keys.is_empty() || !draft.image_indexes.is_empty() {
            let mut image = current.image().to_vec();
            if draft.clear_image {
                removed_object_keys.extend(image.iter().cloned());
                image.clear();
            } else {
                let mut resolved_image_keys = draft.image_keys.clone();
                for index in &draft.image_indexes {
                    let Some(key) = current.image().get(*index) else {
                        return Err(ApplicationError::InvalidInput(format!(
                            "image index out of range: {index}"
                        )));
                    };
                    resolved_image_keys.push(key.clone());
                }
                let remove_set: HashSet<&str> =
                    resolved_image_keys.iter().map(|k| k.as_str()).collect();
                image.retain(|k| {
                    let should_remove = remove_set.contains(k.as_str());
                    if should_remove {
                        removed_object_keys.push(k.clone());
                    }
                    !should_remove
                });
            }
            update.image = Some(image);
        }

        let updated = self.quote_port.update(update).await?;

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
