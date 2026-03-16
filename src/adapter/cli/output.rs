use crate::application::CliImageMode;
use crate::application::service::template::RenderQuoteTemplateService;
use std::io::IsTerminal;
use viuer::{Config as ViuerConfig, print as print_image};

pub(super) async fn print_quote(
    quote: &crate::domain::entity::Quote,
    format: Option<&str>,
    render_template_service: &RenderQuoteTemplateService<'_>,
    image_mode: CliImageMode,
) -> anyhow::Result<()> {
    if let Some(raw) = format {
        if matches!(image_mode, CliImageMode::View) {
            if let Some(target) = extract_single_image_target(raw) {
                if try_print_image_view(render_template_service, quote, target).await? {
                    return Ok(());
                }
            }
        }
        let rendered = render_template_service.execute(quote, raw).await?;
        println!("{rendered}");
    } else {
        println!("{}", serde_json::to_string_pretty(quote)?);
    }
    Ok(())
}

pub(super) async fn print_quotes(
    quotes: &[crate::domain::entity::Quote],
    format: Option<&str>,
    render_template_service: &RenderQuoteTemplateService<'_>,
    image_mode: CliImageMode,
) -> anyhow::Result<()> {
    if let Some(raw) = format {
        for quote in quotes {
            if matches!(image_mode, CliImageMode::View) {
                if let Some(target) = extract_single_image_target(raw) {
                    if try_print_image_view(render_template_service, quote, target).await? {
                        continue;
                    }
                }
            }
            let rendered = render_template_service.execute(quote, raw).await?;
            println!("{rendered}");
        }
    } else {
        println!("{}", serde_json::to_string_pretty(quotes)?);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
enum ImageTemplateTarget {
    Index(usize),
}

/// 解析“仅包含一个模板表达式”的图片目标。
///
/// 支持格式：`{{$image.<index>}}`。
/// 若模板包含额外文本、不是 `$image` 表达式、或索引非法，则返回 `None`。
fn extract_single_image_target(raw_template: &str) -> Option<ImageTemplateTarget> {
    let expr = raw_template.trim();
    if !expr.starts_with("{{") || !expr.ends_with("}}") {
        return None;
    }
    let inner = expr[2..expr.len() - 2].trim();
    let path = inner.strip_prefix('$')?;
    let mut parts = path.split('.').filter(|v| !v.is_empty());
    let head = parts.next()?;
    if head != "image" {
        return None;
    }
    let index_raw = parts.next()?;
    if parts.next().is_some() {
        return None;
    }
    index_raw
        .parse::<usize>()
        .ok()
        .map(ImageTemplateTarget::Index)
}

/// 在 `view` 模式下尝试直接向终端输出图片。
///
/// 行为：
/// - 非 TTY 场景直接返回 `Ok(false)`。
/// - 仅处理单张图片目标（由 `extract_single_image_target` 保证）。
/// - 终端直出失败时返回 `Ok(false)`，由上层回退到文本渲染。
async fn try_print_image_view(
    render_template_service: &RenderQuoteTemplateService<'_>,
    quote: &crate::domain::entity::Quote,
    target: ImageTemplateTarget,
) -> anyhow::Result<bool> {
    if !std::io::stdout().is_terminal() {
        return Ok(false);
    }

    let cfg = ViuerConfig {
        transparent: true,
        ..Default::default()
    };

    let mut printed = false;
    match target {
        ImageTemplateTarget::Index(index) => {
            let Some(bytes) = render_template_service
                .load_image_bytes(quote, index)
                .await?
            else {
                return Ok(false);
            };
            let Ok(img) = image::load_from_memory(&bytes) else {
                return Ok(false);
            };
            if print_image(&img, &cfg).is_ok() {
                printed = true;
            }
        }
    }

    Ok(printed)
}
