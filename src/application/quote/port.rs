use crate::application::ApplicationError;
use crate::application::quote::{QuoteCreate, QuoteQuery, QuoteUpdate};
use crate::domain::entity::Quote;
use async_trait::async_trait;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait QuotePort {
    /// 创建一条 Quote 并返回创建后的完整实体。
    ///
    /// # Parameters
    /// - `create.inline`：多语言内联文本，key 必须是合法 `Lang`。
    /// - `create.external`：多语言外部文本对象 key，key 必须是合法 `Lang`，value 必须是合法 `ObjectKey`。
    /// - `create.markdown`：多语言 markdown 对象 key，key 必须是合法 `Lang`，value 必须是合法 `ObjectKey`。
    /// - `create.image`：图片对象 key 数组，每一项都必须是合法 `ObjectKey`。
    /// - `create.remark`：可选备注。
    /// - 内容约束：`inline/external/markdown/image` 至少要有一种非空。
    ///
    /// # Errors
    /// - 参数不合法时返回 `ApplicationError::InvalidInput` 或 `ApplicationError::Domain(_)`。
    /// - 持久化失败时返回实现层映射后的依赖错误（如 `ApplicationError::Dependency`）。
    async fn create(&self, create: QuoteCreate) -> Result<Quote, ApplicationError>;

    /// 查询单条 Quote。
    ///
    /// # Parameters
    /// - `query.id`
    ///   - `Some(id)`：按主键查询；`id` 必须大于 0。
    ///   - `None`：实现层可返回一条记录（例如随机一条）。
    /// - `query.filter`：存在性条件。
    ///   - `all_of`：子条件全部满足（AND）。
    ///   - `any_of`：子条件任意满足一个（OR）。
    ///   - `not`：对子条件取反（NOT）。
    ///   - `inline_all`：要求同时存在这些语言的 inline。
    ///   - `inline_any`：要求至少存在一个语言的 inline。
    ///   - `external_all/external_any`、`markdown_all/markdown_any`：语义与 inline 一致。
    ///   - `image_exists`：是否要求存在 image 数据。
    ///
    /// # Errors
    /// - 未命中记录时应返回 `ApplicationError::NotFound`（或 `ApplicationError::Domain(DomainError::QuoteNotFound)`）。
    /// - 查询失败时返回实现层映射后的错误。
    async fn get(&self, query: QuoteQuery) -> Result<Quote, ApplicationError>;

    /// 查询多条 Quote。
    ///
    /// # Parameters
    /// - `query.limit`：可选条数上限；如传入，要求大于 0。
    /// - `query.offset`：可选偏移量；如传入，要求大于等于 0。
    /// - `query.filter`：存在性条件，语义与 `get` 相同。
    ///
    /// # Returns
    /// - 返回匹配记录列表；无匹配时返回空数组（而非错误）。
    ///
    /// # Errors
    /// - 参数或查询执行失败时返回实现层映射后的错误。
    async fn list(&self, query: QuoteQuery) -> Result<Vec<Quote>, ApplicationError>;

    /// 统计满足查询条件的 Quote 条数。
    ///
    /// # Parameters
    /// - `query.id`：可选主键筛选。
    /// - `query.filter`：存在性条件，语义与 `get/list` 相同。
    ///
    /// # Returns
    /// - 返回匹配条数（`>= 0`）。
    ///
    /// # Errors
    /// - 参数或查询执行失败时返回实现层映射后的错误。
    async fn count(&self, query: QuoteQuery) -> Result<i64, ApplicationError>;

    /// 更新一条 Quote 并返回更新后的完整实体。
    ///
    /// # Parameters
    /// - `update.id`：必填，目标主键，必须大于 0。
    /// - `update.inline`：可选，传入时表示覆盖该字段。
    /// - `update.external`：可选，传入时表示覆盖该字段。
    /// - `update.markdown`：可选，传入时表示覆盖该字段。
    /// - `update.image`：可选，传入时表示覆盖该字段。
    /// - `update.remark`：可选二层 Option。
    ///   - `None`：不修改 remark。
    ///   - `Some(None)`：清空 remark。
    ///   - `Some(Some(v))`：更新为新值。
    /// - 字段值要求：传入的 `Lang/ObjectKey` 必须已经通过上层校验。
    ///
    /// # Errors
    /// - 记录不存在时返回 `ApplicationError::NotFound`（或领域等价错误）。
    /// - 更新失败时返回实现层映射后的错误。
    async fn update(&self, update: QuoteUpdate) -> Result<Quote, ApplicationError>;

    /// 按主键删除一条 Quote。
    ///
    /// # Parameters
    /// - `id`：目标主键，必须大于 0。
    ///
    /// # Errors
    /// - 记录不存在时可返回 `ApplicationError::NotFound`（或实现层等价语义）。
    /// - 删除失败时返回实现层映射后的错误。
    async fn delete(&self, id: i64) -> Result<(), ApplicationError>;
}
