use async_trait::async_trait;
use crate::app::app_error::AppError;
use crate::quote::{Quote, QuoteAdd, QuoteFilePayload, QuoteQuery};
use serde_json::Value;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait QuotePort {
    /// ***Brief***: 上传对象到外部存储并返回对象 key
    ///
    /// # Parameters
    /// - `path`: 存储路径前缀（如 `text/en`、`image`）
    /// - `payload`: 文件名和字节内容
    /// - `content_type`: MIME 类型（如 `text/plain`, `image/png`, `audio/mpeg`）
    async fn upload_object(
        &self,
        path: &str,
        payload: QuoteFilePayload,
        content_type: &str,
    ) -> Result<String, AppError>;

    /// ***Brief***: 新增一条 Quote 数据并返回写入后的实体
    ///
    /// # Parameters
    /// - `add.content`: quote 内容（JSON object），顶层只允许以下 key：
    ///   - `inline`: `{lang: text}`，文本直接写入 PGSQL。
    ///   - `external`: `{lang: key}`，key 指向 MinIO 中的外部文本对象。
    ///   - `markdown`: `{lang: key}`，key 指向 MinIO 中的 markdown 对象。
    ///   - `image`: `[key, ...]`，key 指向 MinIO 中的图片对象。
    ///   - `audio`: `{group: [key, ...]}`，key 指向 MinIO 中的音频对象。
    /// - `add.content` 允许只包含任意一个类型，也允许多个类型组合。
    /// - `add.active`: 可选激活状态，未传时由存储层使用默认值
    /// - `add.remark`: 可选备注
    async fn add(&self, add: QuoteAdd) -> Result<Quote, AppError>;

    /// ***Brief***: 按 id 覆盖更新 quote.content，并返回更新后的实体
    ///
    /// # Parameters
    /// - `id`: quote 主键 id
    /// - `content`: 完整 content JSON（不是增量 patch）
    async fn update_content(&self, id: i64, content: Value) -> Result<Quote, AppError>;

    /// ***Brief***: 返回一条 Quote 数据
    ///
    /// # Parameters
    /// - `query.id`
    ///     - `Some(id)`: 根据 id 的值返回 Quote
    ///         - *此时会忽略 `query.filter` 的条件*
    ///     - `None`: 随机返回一条 Quote
    ///         - 随机返回的 Quote 需满足 `query.filter` 的条件
    /// - `query.filter`: 条件查询
    /// - `query.active`
    ///     - `None` - 查询所有条目。
    ///     - `Some(true)` - 只查询 `active=true` 的条目。
    ///     - `Some(false)` - 只查询 `active=false` 的条目。
    async fn get(&self, query: QuoteQuery) -> Result<Quote, AppError>;

    /// ***Brief***: 返回多条 Quote 数据
    /// # Parameters
    /// - `query.filter`: 条件查询
    /// - `query.active`
    ///     - `None` - 查询所有条目。
    ///     - `Some(true)` - 只查询 `active=true` 的条目。
    ///     - `Some(false)` - 只查询 `active=false` 的条目。
    /// - `query.limit`: 查询多少条数据。
    /// - `query.offset`：偏移量
    async fn list(&self, query: QuoteQuery) -> Result<Vec<Quote>, AppError>;
}
