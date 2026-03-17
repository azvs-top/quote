use crate::domain::value::{Lang, ObjectKey};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type MultiLangText = HashMap<Lang, String>;
pub type MultiLangObject = HashMap<Lang, ObjectKey>;

/// Quote 聚合根。
///
/// 表示一条已经持久化、满足领域约束的引用记录。
/// 该类型承载：
/// - 核心业务字段
/// - 创建后的不变量校验
/// - 基于补丁的领域更新行为
///
/// 注意：
/// - `id` 必须是有效持久化标识。
/// - `inline/external/markdown/image` 至少要有一种内容存在。
/// - 对字段的任何修改都应通过领域方法完成，以保持约束一致。
#[derive(Debug, Clone, Serialize)]
pub struct Quote {
    id: i64,
    inline: MultiLangText,
    external: MultiLangObject,
    markdown: MultiLangObject,
    image: Vec<ObjectKey>,
    remark: Option<String>,
}

/// Quote 的未持久化草稿。
///
/// 用于“创建一条 Quote”时在进入仓储前承载业务内容。
/// 它只表示“内容是否合法”，不包含数据库主键，因此不能替代 `Quote`。
///
/// 典型使用场景：
/// - 应用服务完成上传、组装对象 key 后
/// - 调用仓储创建记录前
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteDraft {
    inline: MultiLangText,
    external: MultiLangObject,
    markdown: MultiLangObject,
    image: Vec<ObjectKey>,
    remark: Option<String>,
}

/// Quote 的领域补丁。
///
/// 用于表达“对现有 Quote 做哪些变更”。
/// 它不携带完整的新实体，而是携带一组增量操作，例如：
/// - 为 `inline/external/markdown` 按语言新增或覆盖条目
/// - 清空某一类内容
/// - 删除指定语言条目
/// - 为 `image` 追加图片或删除指定对象
/// - 设置或清空 `remark`
///
/// 该类型本身会校验补丁字段的局部合法性；
/// 真正的整体合法性由 `Quote::apply` 在合并旧值后统一校验。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QuotePatch {
    upsert_inline: Option<MultiLangText>,
    clear_inline: bool,
    remove_inline: Vec<Lang>,
    upsert_external: Option<MultiLangObject>,
    clear_external: bool,
    remove_external: Vec<Lang>,
    upsert_markdown: Option<MultiLangObject>,
    clear_markdown: bool,
    remove_markdown: Vec<Lang>,
    append_image: Option<Vec<ObjectKey>>,
    clear_image: bool,
    remove_image: Vec<ObjectKey>,
    remark: Option<Option<String>>,
}

/// 持久化/传输层使用的 Quote 数据结构。
///
/// 注意：
/// - 该类型允许反序列化。
/// - 转换为领域实体必须通过 `TryFrom<QuoteDto> for Quote`，以执行领域校验。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteDto {
    pub id: i64,
    pub inline: MultiLangText,
    pub external: MultiLangObject,
    pub markdown: MultiLangObject,
    pub image: Vec<ObjectKey>,
    pub remark: Option<String>,
}

mod entity;
mod draft;
mod dto;
mod filter;
mod patch;

pub use filter::QuoteFilter;

#[cfg(test)]
mod tests;
