use crate::domain::value::Lang;

/// Quote 的领域筛选条件。
///
/// 该类型描述“什么样的 Quote 算匹配”，属于领域查询语义，
/// 不关心具体由数据库、内存还是其他存储来执行。
///
/// 目前支持：
/// - `all_of` / `any_of` / `not` 组合逻辑
/// - 按 `inline/external/markdown` 是否包含指定语言进行筛选
/// - 按 `image` 是否存在进行筛选
///
/// 注意：
/// - 该类型只表达语义，不负责分页、排序、随机策略等读取控制。
/// - 具体如何翻译成 SQL 或其他查询语法，由基础设施层负责。
#[derive(Debug, Clone, Default)]
pub struct QuoteFilter {
    /// 子条件全部满足（AND）。
    pub all_of: Vec<QuoteFilter>,
    /// 子条件任意满足一个（OR）。
    pub any_of: Vec<QuoteFilter>,
    /// 子条件取反（NOT）。
    pub not: Option<Box<QuoteFilter>>,
    /// 要求 inline 同时包含这些语言。
    pub inline_all: Vec<Lang>,
    /// 要求 inline 至少包含这些语言中的一个。
    pub inline_any: Vec<Lang>,
    /// 要求 external 同时包含这些语言。
    pub external_all: Vec<Lang>,
    /// 要求 external 至少包含这些语言中的一个。
    pub external_any: Vec<Lang>,
    /// 要求 markdown 同时包含这些语言。
    pub markdown_all: Vec<Lang>,
    /// 要求 markdown 至少包含这些语言中的一个。
    pub markdown_any: Vec<Lang>,
    /// 是否要求存在 image 数据。
    pub image_exists: Option<bool>,
}

impl QuoteFilter {
    /// 判断该筛选条件是否为空。
    ///
    /// 空筛选表示“对 Quote 不施加任何约束”。
    pub fn is_empty(&self) -> bool {
        self.all_of.is_empty()
            && self.any_of.is_empty()
            && self.not.is_none()
            && self.inline_all.is_empty()
            && self.inline_any.is_empty()
            && self.external_all.is_empty()
            && self.external_any.is_empty()
            && self.markdown_all.is_empty()
            && self.markdown_any.is_empty()
            && self.image_exists.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::QuoteFilter;
    use crate::domain::value::Lang;

    #[test]
    fn quote_filter_empty_state_belongs_to_domain() {
        assert!(QuoteFilter::default().is_empty());

        let mut filter = QuoteFilter::default();
        filter.inline_any.push(Lang::new("en").expect("valid lang"));

        assert!(!filter.is_empty());
    }
}
