use crate::domain::{
    entity::{MultiLangObject, MultiLangText},
    value::{Lang, ObjectKey},
};

#[derive(Debug, Clone, Default)]
pub struct QuoteCreate {
    pub inline: MultiLangText,
    pub external: MultiLangObject,
    pub markdown: MultiLangObject,
    pub image: Vec<ObjectKey>,
    pub remark: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct QuoteUpdate {
    pub id: i64,
    pub inline: Option<MultiLangText>,
    pub external: Option<MultiLangObject>,
    pub markdown: Option<MultiLangObject>,
    pub image: Option<Vec<ObjectKey>>,
    pub remark: Option<Option<String>>,
}

#[derive(Debug, Clone, Default)]
pub struct QuoteQuery {
    id: Option<i64>,
    limit: Option<i64>,
    offset: Option<i64>,
    filter: QuoteFilter,
}

impl QuoteQuery {
    pub fn builder() -> QuoteQueryBuilder {
        QuoteQueryBuilder::default()
    }

    pub fn id(&self) -> Option<i64> {
        self.id
    }

    pub fn limit(&self) -> Option<i64> {
        self.limit
    }

    pub fn offset(&self) -> Option<i64> {
        self.offset
    }

    pub fn filter(&self) -> &QuoteFilter {
        &self.filter
    }
}

#[derive(Debug, Clone, Default)]
pub struct QuoteFilter {
    /// 子条件全部满足（AND）。
    pub all_of: Vec<QuoteFilter>,
    /// 子条件任意满足一个（OR）。
    pub any_of: Vec<QuoteFilter>,
    /// 子条件取反（NOT）。
    pub not: Option<Box<QuoteFilter>>,

    pub inline_all: Vec<Lang>,
    pub inline_any: Vec<Lang>,
    pub external_all: Vec<Lang>,
    pub external_any: Vec<Lang>,
    pub markdown_all: Vec<Lang>,
    pub markdown_any: Vec<Lang>,
    pub image_exists: Option<bool>,
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

    pub fn filter(mut self, filter: QuoteFilter) -> Self {
        self.inner.filter = filter;
        self
    }

    pub fn with_filter(mut self, filter: Option<QuoteFilter>) -> Self {
        self.inner.filter = filter.unwrap_or_default();
        self
    }

    pub fn inline_all(mut self, langs: Vec<Lang>) -> Self {
        self.inner.filter.inline_all = langs;
        self
    }

    pub fn inline_any(mut self, langs: Vec<Lang>) -> Self {
        self.inner.filter.inline_any = langs;
        self
    }

    pub fn external_all(mut self, langs: Vec<Lang>) -> Self {
        self.inner.filter.external_all = langs;
        self
    }

    pub fn external_any(mut self, langs: Vec<Lang>) -> Self {
        self.inner.filter.external_any = langs;
        self
    }

    pub fn markdown_all(mut self, langs: Vec<Lang>) -> Self {
        self.inner.filter.markdown_all = langs;
        self
    }

    pub fn markdown_any(mut self, langs: Vec<Lang>) -> Self {
        self.inner.filter.markdown_any = langs;
        self
    }

    pub fn image_exists(mut self, exists: bool) -> Self {
        self.inner.filter.image_exists = Some(exists);
        self
    }

    pub fn filter_all_of(mut self, filters: Vec<QuoteFilter>) -> Self {
        self.inner.filter.all_of = filters;
        self
    }

    pub fn filter_any_of(mut self, filters: Vec<QuoteFilter>) -> Self {
        self.inner.filter.any_of = filters;
        self
    }

    pub fn filter_not(mut self, filter: QuoteFilter) -> Self {
        self.inner.filter.not = Some(Box::new(filter));
        self
    }

    pub fn build(self) -> QuoteQuery {
        self.inner
    }
}
