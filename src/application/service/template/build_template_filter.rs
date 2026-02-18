use super::NormalizeTemplateService;
use crate::application::quote::QuoteFilter;
use crate::application::ApplicationError;
use crate::domain::value::Lang;
use std::collections::HashSet;

/// 从模板表达式推导随机查询过滤条件。
pub struct BuildQuoteTemplateFilterService;

impl BuildQuoteTemplateFilterService {
    /// 从原始模板推导随机查询过滤条件。
    ///
    /// 目标：
    /// - 在随机查询前先筛掉明显不匹配模板字段的 Quote，
    ///   减少渲染时大量空值结果。
    pub fn execute(raw_template: &str) -> Result<Option<QuoteFilter>, ApplicationError> {
        let template = NormalizeTemplateService::execute(raw_template)?;
        let exprs = extract_template_exprs(&template);
        if exprs.is_empty() {
            return Ok(None);
        }

        let mut inline_all: HashSet<Lang> = HashSet::new();
        let mut external_all: HashSet<Lang> = HashSet::new();
        let mut markdown_all: HashSet<Lang> = HashSet::new();
        let mut image_exists = false;

        for expr in exprs {
            let normalized = expr
                .trim()
                .trim_start_matches('.')
                .trim_start_matches('$')
                .to_string();
            if normalized.is_empty() {
                continue;
            }

            let mut parts = normalized.split('.');
            let Some(head) = parts.next() else {
                continue;
            };
            let second = parts.next();

            match head {
                "inline" => {
                    if let Some(lang) = second {
                        inline_all.insert(Lang::new(lang.to_string())?);
                    }
                }
                "external" => {
                    if let Some(lang) = second {
                        external_all.insert(Lang::new(lang.to_string())?);
                    }
                }
                "markdown" => {
                    if let Some(lang) = second {
                        markdown_all.insert(Lang::new(lang.to_string())?);
                    }
                }
                // `{{.image}}` / `{{.image.0}}` 都要求至少有 image。
                "image" => image_exists = true,
                _ => {}
            }
        }

        if inline_all.is_empty()
            && external_all.is_empty()
            && markdown_all.is_empty()
            && !image_exists
        {
            return Ok(None);
        }

        let mut filter = QuoteFilter::default();
        filter.inline_all = inline_all.into_iter().collect();
        filter.external_all = external_all.into_iter().collect();
        filter.markdown_all = markdown_all.into_iter().collect();
        if image_exists {
            filter.image_exists = Some(true);
        }
        Ok(Some(filter))
    }
}

/// 提取模板中的表达式内容。
///
/// 例如：`"{{.id}} {{$external.en}}"` -> `[".id", "$external.en"]`
fn extract_template_exprs(template: &str) -> Vec<String> {
    let mut exprs = Vec::new();
    let mut cursor = 0usize;

    loop {
        let remain = &template[cursor..];
        let Some(start_rel) = remain.find("{{") else {
            break;
        };
        let start = cursor + start_rel;
        let after_start = start + 2;
        let Some(end_rel) = template[after_start..].find("}}") else {
            break;
        };
        let end = after_start + end_rel;
        let expr = template[after_start..end].trim();
        if !expr.is_empty() {
            exprs.push(expr.to_string());
        }
        cursor = end + 2;
    }

    exprs
}

#[cfg(test)]
mod tests {
    use super::BuildQuoteTemplateFilterService;

    #[test]
    fn build_filter_returns_none_when_no_supported_expr() {
        let result = BuildQuoteTemplateFilterService::execute("{{.id}}")
            .expect("filter build should not fail");
        assert!(result.is_none());
    }

    #[test]
    fn build_filter_collects_lang_requirements_from_dot_and_dollar_expr() {
        let filter = BuildQuoteTemplateFilterService::execute(
            "{{.inline.en}} {{$external.zh}} {{.markdown.ja}} {{.image.0}}",
        )
        .expect("filter build should succeed")
        .expect("filter should be present");

        assert!(filter.inline_all.iter().any(|lang| lang.as_str() == "en"));
        assert!(filter.external_all.iter().any(|lang| lang.as_str() == "zh"));
        assert!(filter.markdown_all.iter().any(|lang| lang.as_str() == "ja"));
        assert_eq!(filter.image_exists, Some(true));
    }
}
