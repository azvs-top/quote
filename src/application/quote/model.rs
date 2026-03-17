use crate::domain::{value::Lang, QuoteFilter};

/// Quote 的应用层查询对象。
///
/// 该类型用于应用服务向仓储表达一次“读取请求”：
/// - `id`：按主键读取指定 Quote
/// - `filter`：附加领域筛选条件
/// - `limit/offset`：分页控制
///
/// 它属于应用层而非领域层，因为它同时组合了：
/// - 领域筛选语义（`QuoteFilter`）
/// - 读取策略参数（分页、按 id 定位）
///
/// 因此，`QuoteQuery` 负责组织一次查询请求，
/// 但不定义 Quote 本身的业务规则。
#[derive(Debug, Clone, Default)]
pub struct QuoteQuery {
    id: Option<i64>,
    limit: Option<i64>,
    offset: Option<i64>,
    filter: QuoteFilter,
}

impl QuoteQuery {
    /// 创建 `QuoteQuery` 构造器。
    pub fn builder() -> QuoteQueryBuilder {
        QuoteQueryBuilder::default()
    }

    /// 返回按主键查询的目标 id。
    pub fn id(&self) -> Option<i64> {
        self.id
    }

    /// 返回分页上限。
    pub fn limit(&self) -> Option<i64> {
        self.limit
    }

    /// 返回分页偏移量。
    pub fn offset(&self) -> Option<i64> {
        self.offset
    }

    /// 返回附加的领域筛选条件。
    pub fn filter(&self) -> &QuoteFilter {
        &self.filter
    }
}

#[derive(Default)]
pub struct QuoteQueryBuilder {
    inner: QuoteQuery,
}

impl QuoteQueryBuilder {
    /// 指定按主键查询某条 Quote。
    pub fn id(mut self, id: i64) -> Self {
        self.inner.id = Some(id);
        self
    }

    /// 以可选值方式设置 id 条件。
    pub fn with_id(mut self, id: Option<i64>) -> Self {
        self.inner.id = id;
        self
    }

    /// 设置分页上限。
    pub fn limit(mut self, limit: i64) -> Self {
        self.inner.limit = Some(limit);
        self
    }

    /// 以可选值方式设置分页上限。
    pub fn with_limit(mut self, limit: Option<i64>) -> Self {
        self.inner.limit = limit;
        self
    }

    /// 设置分页偏移量。
    pub fn offset(mut self, offset: i64) -> Self {
        self.inner.offset = Some(offset);
        self
    }

    /// 以可选值方式设置分页偏移量。
    pub fn with_offset(mut self, offset: Option<i64>) -> Self {
        self.inner.offset = offset;
        self
    }

    /// 直接替换整个领域筛选条件。
    pub fn filter(mut self, filter: QuoteFilter) -> Self {
        self.inner.filter = filter;
        self
    }

    /// 以可选值方式设置筛选条件；未提供时使用空筛选。
    pub fn with_filter(mut self, filter: Option<QuoteFilter>) -> Self {
        self.inner.filter = filter.unwrap_or_default();
        self
    }

    /// 要求 inline 同时包含给定语言集合。
    pub fn inline_all(mut self, langs: Vec<Lang>) -> Self {
        self.inner.filter.inline_all = langs;
        self
    }

    /// 要求 inline 至少包含给定语言集合中的一个。
    pub fn inline_any(mut self, langs: Vec<Lang>) -> Self {
        self.inner.filter.inline_any = langs;
        self
    }

    /// 要求 external 同时包含给定语言集合。
    pub fn external_all(mut self, langs: Vec<Lang>) -> Self {
        self.inner.filter.external_all = langs;
        self
    }

    /// 要求 external 至少包含给定语言集合中的一个。
    pub fn external_any(mut self, langs: Vec<Lang>) -> Self {
        self.inner.filter.external_any = langs;
        self
    }

    /// 要求 markdown 同时包含给定语言集合。
    pub fn markdown_all(mut self, langs: Vec<Lang>) -> Self {
        self.inner.filter.markdown_all = langs;
        self
    }

    /// 要求 markdown 至少包含给定语言集合中的一个。
    pub fn markdown_any(mut self, langs: Vec<Lang>) -> Self {
        self.inner.filter.markdown_any = langs;
        self
    }

    /// 设置是否要求存在图片。
    pub fn image_exists(mut self, exists: bool) -> Self {
        self.inner.filter.image_exists = Some(exists);
        self
    }

    /// 设置一组必须全部满足的子筛选条件。
    pub fn filter_all_of(mut self, filters: Vec<QuoteFilter>) -> Self {
        self.inner.filter.all_of = filters;
        self
    }

    /// 设置一组任意满足其一的子筛选条件。
    pub fn filter_any_of(mut self, filters: Vec<QuoteFilter>) -> Self {
        self.inner.filter.any_of = filters;
        self
    }

    /// 设置需要取反的子筛选条件。
    pub fn filter_not(mut self, filter: QuoteFilter) -> Self {
        self.inner.filter.not = Some(Box::new(filter));
        self
    }

    /// 构造最终查询对象。
    pub fn build(self) -> QuoteQuery {
        self.inner
    }
}
