use async_trait::async_trait;
use crate::app::app_error::AppError;
use crate::dict::{Dict, DictQuery, DictType};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait DictPort {
    /// ***Brief***: 获取单个字典类型
    ///
    /// # Behavior
    /// - 仅通过 `query.type_id` 或 `query.type_key` 查询。
    /// - 若 `type_id` 与 `type_key` 同时传入，以 `type_id` 为准。
    ///
    /// # Parameters
    /// - `query.type_id` 或 `query.type_key`: 指定具体类型。
    async fn get_type(&self, query: DictQuery) -> Result<DictType, AppError>;

    /// ***Brief***: 列出所有字典类型（去重后的 type 视图）
    ///
    /// # Behavior
    /// - 数据来源为 `quote.f_dict(langs)` 的去重结果，不直接查询 dict_type 表。
    /// - 返回每个类型的元信息（type_id/type_key/type_creator/active/remark）。
    ///
    /// # Parameters
    /// - `query.langs`: 传给 `f_dict(langs)`，决定 `item_value` 的语言优先级。
    /// - `query.type_id` / `query.type_key`: 用于过滤具体类型。
    /// - `query.type_creator`: 过滤类型创建者。
    /// - `query.active`: 过滤 active 状态。
    /// - `query.limit` / `query.offset`: 分页。
    async fn list_type(&self, query: DictQuery) -> Result<Vec<DictType>, AppError>;

    /// ***Brief***: 列出某个类型下的所有 item
    ///
    /// # Behavior
    /// - 若传入 `type_id`：返回该 `type_id` 下的所有 item。
    /// - 若传入 `type_key`：返回该 `type_key` 下的所有 item。
    /// - 若 `type_id` 与 `type_key` 同时传入，以 `type_id` 为准。
    ///
    /// # Parameters
    /// - `query.langs`: 传给 `f_dict(langs)`，决定 `display_text` 的语言优先级。
    /// - `query.type_id` / `query.type_key`: 限定字典类型范围。
    /// - `query.item_creator`: 过滤 item 创建者。
    /// - `query.is_default`: 过滤默认项。
    /// - `query.active`: 过滤 active 状态。
    /// - `query.limit` / `query.offset`: 分页。
    async fn list_item(&self, query: DictQuery) -> Result<Vec<Dict>, AppError>;

    /// ***Brief***: 获取单个 item（按类型 + item key）
    ///
    /// # Parameters
    /// - `query.langs`: 传给 `f_dict(langs)`，决定 `display_text` 的语言优先级。
    /// - `query.type_id` 或 `query.type_key`: 指定字典类型。
    /// - `query.item_id` 或 `query.item_key`: 指定具体 item。
    /// - `query.active`: 过滤 active 状态（可选）。
    async fn get_item(&self, query: DictQuery) -> Result<Dict, AppError>;
}
