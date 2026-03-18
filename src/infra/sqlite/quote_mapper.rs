use crate::application::ApplicationError;
use crate::domain::quote::{MultiLangObject, MultiLangText, Quote, QuoteDraft};
use crate::domain::value::ObjectKey;
use sqlx::FromRow;

/// SQLite 持久化映射器。
///
/// 职责：
/// - 定义 SQLite 查询结果对应的行结构
/// - 负责领域对象与 JSON 文本字段之间的编解码
/// - 将数据库行恢复为领域 `Quote`
///
/// 不负责：
/// - SQL 拼装
/// - 查询条件翻译
/// - 仓储流程控制
#[derive(Debug, FromRow)]
pub struct QuoteRow {
    /// Quote 主键。
    pub id: i64,
    /// inline 字段的 JSON 文本。
    pub inline: String,
    /// external 字段的 JSON 文本。
    pub external: String,
    /// markdown 字段的 JSON 文本。
    pub markdown: String,
    /// image 字段的 JSON 文本。
    pub image: String,
    /// remark 原始值。
    pub remark: Option<String>,
}

/// 将任意可序列化对象转换为 SQLite 持久化用的 JSON 文本。
pub fn serialize_json_text<T: serde::Serialize + ?Sized>(
    value: &T,
    field: &str,
) -> Result<String, ApplicationError> {
    serde_json::to_string(value)
        .map_err(|err| ApplicationError::Dependency(format!("serialize {field} failed: {err}")))
}

fn deserialize_json_text<T: serde::de::DeserializeOwned>(
    value: String,
    field: &str,
) -> Result<T, ApplicationError> {
    serde_json::from_str(&value)
        .map_err(|err| ApplicationError::Dependency(format!("deserialize {field} failed: {err}")))
}

/// 将一行 SQLite 记录恢复为领域 `Quote`。
///
/// 恢复过程中会重新走领域构造校验，确保数据库中的数据仍满足领域约束。
pub fn map_row_to_quote(row: QuoteRow) -> Result<Quote, ApplicationError> {
    let inline: MultiLangText = deserialize_json_text(row.inline, "inline")?;
    let external: MultiLangObject = deserialize_json_text(row.external, "external")?;
    let markdown: MultiLangObject = deserialize_json_text(row.markdown, "markdown")?;
    let image: Vec<ObjectKey> = deserialize_json_text(row.image, "image")?;

    Quote::new(row.id, inline, external, markdown, image, row.remark).map_err(ApplicationError::from)
}

/// 将待创建的 `QuoteDraft` 转换为 SQLite 插入语句所需的字段值。
pub fn draft_to_row_values(
    draft: &QuoteDraft,
) -> Result<(String, String, String, String, Option<&str>), ApplicationError> {
    Ok((
        serialize_json_text(draft.inline(), "inline")?,
        serialize_json_text(draft.external(), "external")?,
        serialize_json_text(draft.markdown(), "markdown")?,
        serialize_json_text(draft.image(), "image")?,
        draft.remark(),
    ))
}

/// 将完整 `Quote` 转换为 SQLite 更新语句所需的字段值。
pub fn quote_to_row_values(
    quote: &Quote,
) -> Result<(String, String, String, String, Option<&str>), ApplicationError> {
    Ok((
        serialize_json_text(quote.inline(), "inline")?,
        serialize_json_text(quote.external(), "external")?,
        serialize_json_text(quote.markdown(), "markdown")?,
        serialize_json_text(quote.image(), "image")?,
        quote.remark(),
    ))
}
