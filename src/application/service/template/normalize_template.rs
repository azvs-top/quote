use crate::application::ApplicationError;

/// 标准化并校验 `--format` 模板字符串。
pub struct NormalizeTemplateService;

impl NormalizeTemplateService {
    /// 标准化模板原文。
    ///
    /// 行为：
    /// - 先做最小语法校验（必须包含 `{{` 与 `}}`）。
    /// - 再将常见转义序列还原为真实字符。
    pub fn execute(raw: &str) -> Result<String, ApplicationError> {
        if !raw.contains("{{") || !raw.contains("}}") {
            return Err(ApplicationError::InvalidInput(
                "--format only accepts template strings like '{{.inline.en}}' or '{{$external.en}}'"
                    .to_string(),
            ));
        }
        Ok(unescape_template(raw))
    }
}

/// 将 CLI 输入中的转义字符（如 `\n`）转为真实字符。
///
/// 未识别的转义会保留原样（例如 `\x` -> `\x`）。
fn unescape_template(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut chars = raw.chars();

    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }

        match chars.next() {
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some('t') => out.push('\t'),
            Some('\\') => out.push('\\'),
            Some('"') => out.push('"'),
            Some('\'') => out.push('\''),
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::NormalizeTemplateService;
    use crate::application::ApplicationError;

    #[test]
    fn normalize_rejects_non_template_input() {
        let result = NormalizeTemplateService::execute("plain text");
        assert!(matches!(result, Err(ApplicationError::InvalidInput(_))));
    }

    #[test]
    fn normalize_unescapes_common_sequences() {
        let input = "{{.inline.en}}\\nline2\\t\\\"q\\\"\\\\";
        let result = NormalizeTemplateService::execute(input).expect("should normalize");
        assert_eq!(result, "{{.inline.en}}\nline2\t\"q\"\\");
    }
}
