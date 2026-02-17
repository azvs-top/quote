//! Template 模块调用流程（当前对接 CLI `--format`）：
//!
//! 1) 随机获取前的过滤构建：
//! - `BuildQuoteTemplateFilterService::execute`
//! - -> `NormalizeTemplateService::execute`（校验与转义还原）
//! - -> 提取表达式并构造 `QuoteFilter`
//!
//! 2) 获取到 Quote 后的模板渲染：
//! - `RenderQuoteTemplateService::execute`
//! - -> `NormalizeTemplateService::execute`（同一套输入标准化）
//! - -> 渲染 `{{...}}`
//!   - `.path`：直接读取 Quote 字段（如 `.external.en` 输出 key）
//!   - `$path`：读取扩展对象内容（当前仅 `$external.<lang>`）
//!
//! 说明：
//! - 规则拆分在 `normalize/filter/render` 三个 service，便于后续扩展 `$markdown/$image`。
mod build_template_filter;
mod normalize_template;
mod render_quote_template;

pub use build_template_filter::*;
pub use normalize_template::*;
pub use render_quote_template::*;
