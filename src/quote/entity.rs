use serde_json::Value;
use sqlx::FromRow;
use crate::app::app_error::AppError;
use crate::PgJson;

#[derive(Debug, Clone, FromRow)]
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
    pub fn active(mut self, active: bool) -> Self {
        self.inner.active = Some(active);
        self
    }

    pub fn filter(mut self, filter: QuoteQueryFilter) -> Self {
        self.inner.filter = Some(filter);
        self
    }

    pub fn limit(mut self, limit: i64) -> Self {
        self.inner.limit = Some(limit);
        self
    }

    pub fn offset(mut self, offset: i64) -> Self {
        self.inner.offset = Some(offset);
        self
    }

    pub fn build(self) -> QuoteQuery {
        self.inner
    }
}

#[derive(Debug, Clone)]
pub enum QuoteQueryFilter {
    /// quote.content.inline 同时存在这些语言
    AllLangs(Vec<String>),

    /// quote.content.inline 存在任意一个语言
    AnyLang(Vec<String>),
}