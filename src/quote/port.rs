use async_trait::async_trait;
use crate::app::app_error::AppError;
use crate::quote::{Quote, QuoteQuery};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait QuotePort {

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