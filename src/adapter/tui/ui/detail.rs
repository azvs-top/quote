use crate::adapter::tui::app::TuiApp;
use crate::adapter::tui::ui::common::preview_inline_full;
use crate::domain::entity::Quote;
use ratatui::layout::Rect;
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

/// 渲染“详情页”主视图。
///
/// 特点：
/// - 占用正文区域整屏显示 Detail 内容。
/// - 启用纵向滚动，滚动偏移由 `app.detail_scroll` 控制（j/k）。
pub(super) fn render_single(frame: &mut ratatui::Frame<'_>, area: Rect, app: &TuiApp) {
    let detail = match app.selected_quote() {
        Some(q) => build_detail_full_text(q),
        None => Text::from("No quote selected"),
    };
    frame.render_widget(
        Paragraph::new(detail)
            .block(Block::default().title("Detail").borders(Borders::ALL))
            .scroll((app.detail_scroll, 0))
            .wrap(Wrap { trim: false }),
        area,
    );
}

/// 渲染“列表页右侧”详情预览面板。
///
/// 特点：
/// - 使用精简文本，避免在列表双栏中占用过多空间。
/// - 不启用滚动，保持列表页浏览效率。
pub(super) fn render_preview(frame: &mut ratatui::Frame<'_>, area: Rect, app: &TuiApp) {
    let detail = match app.selected_quote() {
        Some(q) => build_detail_preview_text(q),
        None => Text::from("No quote selected"),
    };
    frame.render_widget(
        Paragraph::new(detail)
            .block(Block::default().title("Detail").borders(Borders::ALL))
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn build_detail_preview_text(quote: &Quote) -> Text<'static> {
    let mut lines = vec![
        Line::from(format!("id: {}", quote.id())),
        Line::from(format!("inline langs: {}", quote.inline().len())),
        Line::from(format!("external langs: {}", quote.external().len())),
        Line::from(format!("markdown langs: {}", quote.markdown().len())),
        Line::from(format!("images: {}", quote.image().len())),
        Line::from(""),
        Line::from("inline:"),
    ];

    let mut inline_items: Vec<_> = quote
        .inline()
        .iter()
        .map(|(lang, text)| (lang.as_str().to_string(), text.clone()))
        .collect();
    inline_items.sort_by(|a, b| a.0.cmp(&b.0));
    if inline_items.is_empty() {
        lines.push(Line::from("  <empty>"));
    } else {
        for (lang, text) in inline_items {
            lines.push(Line::from(format!("  {lang}: {text}")));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(format!(
        "inline preview(en/zh/first): {}",
        preview_inline_full(quote)
    )));
    Text::from(lines)
}

fn build_detail_full_text(quote: &Quote) -> Text<'static> {
    let mut lines = vec![
        Line::from(format!("id: {}", quote.id())),
        Line::from(format!("remark: {}", quote.remark().unwrap_or("<none>"))),
        Line::from(""),
        Line::from(format!("inline ({}):", quote.inline().len())),
    ];

    let mut inline_items: Vec<_> = quote
        .inline()
        .iter()
        .map(|(lang, text)| (lang.as_str().to_string(), text.clone()))
        .collect();
    inline_items.sort_by(|a, b| a.0.cmp(&b.0));
    if inline_items.is_empty() {
        lines.push(Line::from("  <empty>"));
    } else {
        for (lang, text) in inline_items {
            lines.push(Line::from(format!("  {lang}: {text}")));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(format!("external ({}):", quote.external().len())));
    let mut external_items: Vec<_> = quote
        .external()
        .iter()
        .map(|(lang, key)| (lang.as_str().to_string(), key.as_str().to_string()))
        .collect();
    external_items.sort_by(|a, b| a.0.cmp(&b.0));
    if external_items.is_empty() {
        lines.push(Line::from("  <empty>"));
    } else {
        for (lang, key) in external_items {
            lines.push(Line::from(format!("  {lang}: {key}")));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(format!("markdown ({}):", quote.markdown().len())));
    let mut markdown_items: Vec<_> = quote
        .markdown()
        .iter()
        .map(|(lang, key)| (lang.as_str().to_string(), key.as_str().to_string()))
        .collect();
    markdown_items.sort_by(|a, b| a.0.cmp(&b.0));
    if markdown_items.is_empty() {
        lines.push(Line::from("  <empty>"));
    } else {
        for (lang, key) in markdown_items {
            lines.push(Line::from(format!("  {lang}: {key}")));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(format!("images ({}):", quote.image().len())));
    if quote.image().is_empty() {
        lines.push(Line::from("  <empty>"));
    } else {
        for (idx, key) in quote.image().iter().enumerate() {
            lines.push(Line::from(format!("  [{idx}] {}", key.as_str())));
        }
    }

    Text::from(lines)
}
