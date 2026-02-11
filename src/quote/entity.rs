use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use crate::app::app_error::AppError;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Quote {
    pub id: i64,
    pub content: Value,
    pub active: bool,
    pub remark: Option<String>,
}
impl Quote {
    pub fn new(
        id: i64,
        content: Value,
        active: bool,
        remark: Option<String>,
    ) -> Result<Quote, AppError> {
        if content.is_null() {
            return Err(AppError::QuoteInvalidContent);
        }
        Ok(Self {
            id,
            content,
            active,
            remark,
        })
    }

    pub fn get_inline_texts_by_langs(&self, langs: &[String]) -> Result<Vec<String>, AppError> {
        if langs.is_empty() {
            return Err(AppError::QuoteMissingContent);
        }

        let content_obj = self
            .content
            .as_object()
            .ok_or(AppError::QuoteInvalidContent)?;

        let inline_obj = content_obj
            .get("inline")
            .ok_or(AppError::QuoteMissingContent)?
            .as_object()
            .ok_or(AppError::QuoteInvalidContent)?;

        let mut texts = Vec::with_capacity(langs.len());
        for lang in langs {
            let text = inline_obj
                .get(lang)
                .ok_or(AppError::QuoteMissingContent)?
                .as_str()
                .ok_or(AppError::QuoteInvalidContent)?;
            texts.push(text.to_string());
        }

        Ok(texts)
    }
}

#[derive(Default, Debug, Clone)]
pub struct QuoteQuery {
    id: Option<i64>,
    filter: Option<QuoteQueryFilter>,
    active: Option<bool>,
    limit: Option<i64>,
    offset: Option<i64>,
}
impl QuoteQuery {
    pub fn builder() -> QuoteQueryBuilder {
        QuoteQueryBuilder::default()
    }

    pub fn id(&self) -> Option<i64> {
        self.id
    }

    pub fn filter(&self) -> Option<&QuoteQueryFilter> {
        self.filter.as_ref()
    }

    pub fn active(&self) -> Option<bool> {
        self.active
    }

    pub fn limit(&self) -> Option<i64> {
        self.limit
    }

    pub fn offset(&self) -> Option<i64> {
        self.offset
    }
}

#[derive(Default)]
pub struct QuoteQueryBuilder {
    inner: QuoteQuery,
}
impl QuoteQueryBuilder {
    pub fn id(mut self, id: i64) -> Self {
        self.inner.id = Some(id);
        self
    }

    pub fn with_id(mut self, id: Option<i64>) -> Self {
        self.inner.id = id;
        self
    }

    pub fn active(mut self, active: bool) -> Self {
        self.inner.active = Some(active);
        self
    }

    pub fn with_active(mut self, active: Option<bool>) -> Self {
        self.inner.active = active;
        self
    }

    pub fn filter(mut self, filter: QuoteQueryFilter) -> Self {
        self.inner.filter = Some(filter);
        self
    }

    pub fn with_filter(mut self, filter: Option<QuoteQueryFilter>) -> Self {
        self.inner.filter = filter;
        self
    }

    pub fn limit(mut self, limit: i64) -> Self {
        self.inner.limit = Some(limit);
        self
    }

    pub fn with_limit(mut self, limit: Option<i64>) -> Self {
        self.inner.limit = limit;
        self
    }

    pub fn offset(mut self, offset: i64) -> Self {
        self.inner.offset = Some(offset);
        self
    }

    pub fn with_offset(mut self, offset: Option<i64>) -> Self {
        self.inner.offset = offset;
        self
    }

    pub fn build(self) -> QuoteQuery {
        self.inner
    }
}

#[derive(Debug, Clone)]
pub enum QuoteQueryFilter {

    // 组合
    And(Vec<QuoteQueryFilter>),
    Or(Vec<QuoteQueryFilter>),

    // 内容能力
    HasInline,
    HasExternal,
    HasMarkdown,
    HasImage,
    HasAudio,

    /// quote.content.inline 同时存在这些语言
    HasInlineAllLang(Vec<String>),

    /// quote.content.inline 存在任意一个语言
    HasInlineAnyLang(Vec<String>),

    HasExternalAllLang(Vec<String>),

    HasExternalAnyLang(Vec<String>),

    HasMarkdownAllLang(Vec<String>),

    HasMarkdownAnyLang(Vec<String>),
}
