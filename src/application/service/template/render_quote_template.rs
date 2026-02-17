use super::NormalizeTemplateService;
use crate::application::storage::StoragePort;
use crate::application::ApplicationError;
use crate::domain::entity::Quote;
use crate::domain::value::Lang;
use image::imageops::FilterType;
use image::GenericImageView;
use serde_json::Value;

#[derive(Debug, Clone, Copy, Default)]
pub enum TemplateImageMode {
    #[default]
    Meta,
    Ascii,
    View,
}

/// 渲染单条 Quote 的模板字符串。
///
/// 规则：
/// - `.path` 从 Quote JSON 字段读取（例如 `.external.en` 返回 key）。
/// - `$path` 读取扩展数据（当前支持 `$external.<lang>` 与 `$markdown.<lang>`）。
pub struct RenderQuoteTemplateService<'a> {
    storage: &'a (dyn StoragePort + Send + Sync),
    image_mode: TemplateImageMode,
}

impl<'a> RenderQuoteTemplateService<'a> {
    /// 创建渲染服务，注入对象存储访问能力。
    pub fn new(storage: &'a (dyn StoragePort + Send + Sync), image_mode: TemplateImageMode) -> Self {
        Self { storage, image_mode }
    }

    /// 仅加载指定索引的图片原始字节（不做格式化输出）。
    ///
    /// 用于 adapter 层的终端原图预览能力（如 `--image view`）。
    pub async fn load_image_bytes(
        &self,
        quote: &Quote,
        index: usize,
    ) -> Result<Option<Vec<u8>>, ApplicationError> {
        let Some(key) = quote.image().get(index) else {
            return Ok(None);
        };
        let bytes = self.storage.download(key).await?;
        Ok(Some(bytes))
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
    /// 当前支持：
    /// - `$external.<lang>`：下载对象(仅在内存中)并按文本输出。
    /// - `$markdown.<lang>`：下载对象(仅在内存中)并按 markdown 原文输出。
    /// - `$image.<index>`：下载对象(仅在内存中)并输出运行时解析的图片信息。
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
            "markdown" => {
                let Some(lang_raw) = parts.next() else {
                    return Ok(None);
                };
                let lang = Lang::new(lang_raw.to_string())?;
                let Some(key) = quote.markdown().get(&lang) else {
                    return Ok(None);
                };
                let bytes = self.storage.download(key).await?;
                Ok(Some(String::from_utf8_lossy(&bytes).to_string()))
            }
            "image" => {
                // `$image`：输出所有图片；`$image.<index>`：输出单张图片。
                let Some(index_raw) = parts.next() else {
                    let mut rendered_all = Vec::with_capacity(quote.image().len());
                    for key in quote.image() {
                        let bytes = self.storage.download(key).await?;
                        // 对 `$image`（不带索引）固定输出 meta，避免与 CLI 的图片预览参数产生歧义。
                        rendered_all.push(describe_image_bytes(&bytes));
                    }
                    return serde_json::to_string(&rendered_all)
                        .map(Some)
                        .map_err(|err| {
                            ApplicationError::Dependency(format!(
                                "serialize image render result failed: {err}"
                            ))
                        });
                };

                let Ok(index) = index_raw.parse::<usize>() else {
                    return Ok(None);
                };
                let Some(key) = quote.image().get(index) else {
                    return Ok(None);
                };
                let bytes = self.storage.download(key).await?;
                Ok(Some(match self.image_mode {
                    TemplateImageMode::Meta => describe_image_bytes(&bytes),
                    TemplateImageMode::Ascii => render_image_ascii(&bytes, false),
                    TemplateImageMode::View => render_image_ascii(&bytes, true),
                }))
            }
            _ => Ok(None),
        }
    }
}

/// 运行时解析图片信息，输出用于 CLI 展示的概要文本。
///
/// 示例：
/// - `PNG 1920x1080 (245.7 KB)`
/// - `unknown image (12.3 KB)`（无法识别或解码）
fn describe_image_bytes(bytes: &[u8]) -> String {
    let size = human_readable_bytes(bytes.len() as u64);

    let Ok(format) = image::guess_format(bytes) else {
        return format!("unknown image ({size})");
    };
    let format_name = image_format_name(format);

    let Ok(img) = image::load_from_memory(bytes) else {
        return format!("{format_name} ({size})");
    };
    let (width, height) = img.dimensions();
    format!("{format_name} {width}x{height} ({size})")
}

fn render_image_ascii(bytes: &[u8], view_mode: bool) -> String {
    let Ok(img) = image::load_from_memory(bytes) else {
        return describe_image_bytes(bytes);
    };

    let (src_w, src_h) = img.dimensions();
    if src_w == 0 || src_h == 0 {
        return describe_image_bytes(bytes);
    }

    let target_w: u32 = if view_mode { 96 } else { 64 };
    let aspect = src_h as f32 / src_w as f32;
    let target_h = ((target_w as f32 * aspect) * 0.5).round().max(1.0) as u32;

    let gray = img.to_luma8();
    let resized = image::imageops::resize(&gray, target_w, target_h, FilterType::Triangle);
    let charset: &[u8] = if view_mode {
        b" .,-~:;=!*#$@"
    } else {
        b" .:-=+*#%@"
    };

    let mut out = String::new();
    for y in 0..target_h {
        for x in 0..target_w {
            let v = resized.get_pixel(x, y).0[0] as usize;
            let idx = v * (charset.len() - 1) / 255;
            out.push(charset[idx] as char);
        }
        if y + 1 < target_h {
            out.push('\n');
        }
    }
    out
}

fn image_format_name(format: image::ImageFormat) -> &'static str {
    match format {
        image::ImageFormat::Png => "PNG",
        image::ImageFormat::Jpeg => "JPEG",
        image::ImageFormat::Gif => "GIF",
        image::ImageFormat::WebP => "WEBP",
        image::ImageFormat::Bmp => "BMP",
        image::ImageFormat::Tiff => "TIFF",
        image::ImageFormat::Ico => "ICO",
        image::ImageFormat::Avif => "AVIF",
        image::ImageFormat::Pnm => "PNM",
        image::ImageFormat::Tga => "TGA",
        image::ImageFormat::Dds => "DDS",
        image::ImageFormat::Farbfeld => "FARBFELD",
        image::ImageFormat::Qoi => "QOI",
        image::ImageFormat::OpenExr => "OPENEXR",
        image::ImageFormat::Hdr => "HDR",
        _ => "IMAGE",
    }
}

fn human_readable_bytes(size: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];
    let mut value = size as f64;
    let mut unit = 0usize;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        return format!("{size} {}", UNITS[unit]);
    }
    format!("{value:.1} {}", UNITS[unit])
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
