use super::NormalizeTemplateService;
use crate::application::storage::StoragePort;
use crate::application::ApplicationError;
use crate::domain::entity::Quote;
use crate::domain::value::Lang;
use serde_json::Value;

/// 渲染单条 Quote 的模板字符串。
///
/// 规则：
/// - `.path` 从 Quote JSON 字段读取（例如 `.external.en` 返回 key）。
/// - `$path` 读取扩展数据（当前仅支持 `$external.<lang>` 返回对象内容）。
pub struct RenderQuoteTemplateService<'a> {
    storage: &'a (dyn StoragePort + Send + Sync),
}

impl<'a> RenderQuoteTemplateService<'a> {
    /// 创建渲染服务，注入对象存储访问能力。
    pub fn new(storage: &'a (dyn StoragePort + Send + Sync)) -> Self {
        Self { storage }
    }

    /// 将单条 Quote 按模板渲染为字符串。
    ///
    /// 流程：
    /// - 标准化模板输入。
    /// - 将 Quote 转为 JSON 视图（用于 `.path` 读取）。
    /// - 扫描并替换 `{{...}}` 表达式。
    pub async fn execute(
        &self,
        quote: &Quote,
        raw_template: &str,
    ) -> Result<String, ApplicationError> {
        let template = NormalizeTemplateService::execute(raw_template)?;
        let root = serde_json::to_value(quote).map_err(|err| {
            ApplicationError::Dependency(format!("serialize quote for template failed: {err}"))
        })?;

        let mut out = String::new();
        let mut cursor = 0usize;

        loop {
            let remain = &template[cursor..];
            let Some(start_rel) = remain.find("{{") else {
                out.push_str(remain);
                break;
            };
            let start = cursor + start_rel;
            out.push_str(&template[cursor..start]);

            let after_start = start + 2;
            let Some(end_rel) = template[after_start..].find("}}") else {
                out.push_str(&template[start..]);
                break;
            };
            let end = after_start + end_rel;

            let expr = template[after_start..end].trim();
            out.push_str(&self.resolve_expr(expr, quote, &root).await?);

            cursor = end + 2;
        }

        Ok(out)
    }

    /// 解析单个表达式。
    ///
    /// - `$...`：读取扩展对象内容（当前仅 external）。
    /// - `.foo.bar`：读取 Quote JSON 字段。
    async fn resolve_expr(
        &self,
        expr: &str,
        quote: &Quote,
        root: &Value,
    ) -> Result<String, ApplicationError> {
        let expr = expr.trim();
        if expr.starts_with('$') {
            let value = self.resolve_dollar_expr(quote, expr).await?;
            return Ok(value.unwrap_or_default());
        }

        let key = expr.strip_prefix('.').unwrap_or(expr);
        Ok(lookup_template_key(root, key))
    }

    /// 处理 `$` 前缀表达式。
    ///
    /// 当前仅支持：
    /// - `$external.<lang>`：下载对象并按文本输出。
    ///
    /// 返回 `Ok(None)` 代表表达式不支持或目标不存在。
    async fn resolve_dollar_expr(
        &self,
        quote: &Quote,
        expr: &str,
    ) -> Result<Option<String>, ApplicationError> {
        let Some(path) = expr.strip_prefix('$') else {
            return Ok(None);
        };

        let mut parts = path.split('.').filter(|v| !v.is_empty());
        let Some(head) = parts.next() else {
            return Ok(None);
        };

        match head {
            "external" => {
                let Some(lang_raw) = parts.next() else {
                    return Ok(None);
                };
                let lang = Lang::new(lang_raw.to_string())?;
                let Some(key) = quote.external().get(&lang) else {
                    return Ok(None);
                };
                let bytes = self.storage.download(key).await?;
                Ok(Some(String::from_utf8_lossy(&bytes).to_string()))
            }
            _ => Ok(None),
        }
    }
}

/// 从 JSON 结构中按点路径读取值。
///
/// 读取失败时返回空字符串，保持模板渲染幂等（不抛错）。
fn lookup_template_key(root: &Value, key: &str) -> String {
    let mut current = root;
    for segment in key.split('.').filter(|s| !s.is_empty()) {
        match current {
            Value::Object(map) => {
                let Some(next) = map.get(segment) else {
                    return String::new();
                };
                current = next;
            }
            Value::Array(arr) => {
                let Ok(idx) = segment.parse::<usize>() else {
                    return String::new();
                };
                let Some(next) = arr.get(idx) else {
                    return String::new();
                };
                current = next;
            }
            _ => return String::new(),
        }
    }

    match current {
        Value::Null => String::new(),
        Value::Bool(v) => v.to_string(),
        Value::Number(v) => v.to_string(),
        Value::String(v) => v.clone(),
        _ => current.to_string(),
    }
}
