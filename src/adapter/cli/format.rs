use crate::application::CliImageMode;
use crate::application::service::template::TemplateImageMode;
use std::collections::HashMap;

impl From<CliImageMode> for TemplateImageMode {
    fn from(value: CliImageMode) -> Self {
        match value {
            CliImageMode::Meta => TemplateImageMode::Meta,
            CliImageMode::Ascii => TemplateImageMode::Ascii,
            CliImageMode::View => TemplateImageMode::View,
        }
    }
}

/// 将 CLI 层图片参数映射为统一图片渲染模式。
///
/// 规则：
/// - `--image-view` 优先级最高。
/// - 其次是 `--image-ascii`。
/// - 两者都未指定时回退为默认 `meta`。
///
/// 说明：
/// - 该函数属于 adapter 层参数适配，避免将 cli 参数语义泄漏到 application 层。
pub(super) fn resolve_image_mode(
    image_ascii: bool,
    image_view: bool,
    default_mode: CliImageMode,
) -> CliImageMode {
    if image_view {
        return CliImageMode::View;
    }
    if image_ascii {
        return CliImageMode::Ascii;
    }
    default_mode
}

pub(super) fn resolve_effective_format(
    format: Option<&str>,
    preset: Option<&str>,
    default_format: Option<&str>,
    presets: &HashMap<String, String>,
) -> anyhow::Result<Option<String>> {
    if let Some(raw) = format {
        return Ok(Some(raw.to_string()));
    }
    if let Some(preset_name) = preset {
        let value = presets
            .get(preset_name)
            .ok_or_else(|| anyhow::anyhow!("unknown format preset: {preset_name}"))?;
        return Ok(Some(value.clone()));
    }
    Ok(default_format.map(|v| v.to_string()))
}
