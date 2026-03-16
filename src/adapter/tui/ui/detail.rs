use crate::adapter::tui::state::TuiState;
use crate::domain::entity::Quote;
use ratatui::layout::Rect;
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

pub(crate) fn render_single(frame: &mut ratatui::Frame<'_>, area: Rect, state: &TuiState) {
    let (detail, line_count) = match state.selected_quote() {
        Some(quote) => build_detail_full_text(quote),
        None => (Text::from("No quote selected"), 1),
    };
    let visible_lines = area.height.saturating_sub(2) as usize;
    let max_scroll = line_count.saturating_sub(visible_lines) as u16;
    frame.render_widget(
        Paragraph::new(detail)
            .block(Block::default().title("Detail").borders(Borders::ALL))
            .scroll((state.view.detail_scroll.min(max_scroll), 0))
            .wrap(Wrap { trim: false }),
        area,
    );
}

pub(crate) fn render_preview(frame: &mut ratatui::Frame<'_>, area: Rect, state: &TuiState) {
    let detail = match state.selected_quote() {
        Some(quote) => build_detail_preview_text(quote),
        None => Text::from("No quote selected"),
    };
    frame.render_widget(
        Paragraph::new(detail)
            .block(Block::default().title("Preview").borders(Borders::ALL))
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn build_detail_preview_text(quote: &Quote) -> Text<'static> {
    let mut lines = vec![
        Line::from(format!("id: {}", quote.id())),
        Line::from(format!("remark: {}", quote.remark().unwrap_or("<none>"))),
        Line::from(format!("inline langs: {}", quote.inline().len())),
        Line::from(format!("external langs: {}", quote.external().len())),
        Line::from(format!("markdown langs: {}", quote.markdown().len())),
        Line::from(format!("images: {}", quote.image().len())),
        Line::from(""),
    ];
    push_inline_section(&mut lines, quote);
    Text::from(lines)
}

fn build_detail_full_text(quote: &Quote) -> (Text<'static>, usize) {
    let mut lines = vec![
        Line::from(format!("id: {}", quote.id())),
        Line::from(format!("remark: {}", quote.remark().unwrap_or("<none>"))),
        Line::from(""),
    ];
    push_inline_section(&mut lines, quote);
    push_external_section(&mut lines, quote);
    push_markdown_section(&mut lines, quote);
    push_image_section(&mut lines, quote);

    let line_count = lines.len();
    (Text::from(lines), line_count)
}

fn push_inline_section(lines: &mut Vec<Line<'static>>, quote: &Quote) {
    lines.push(Line::from("inline:"));
    let mut inline_items: Vec<_> = quote
        .inline()
        .iter()
        .map(|(lang, text)| (lang.as_str().to_string(), text.clone()))
        .collect();
    inline_items.sort_by(|a, b| a.0.cmp(&b.0));
    push_items(lines, inline_items);
}

fn push_external_section(lines: &mut Vec<Line<'static>>, quote: &Quote) {
    lines.push(Line::from(""));
    lines.push(Line::from("external:"));
    let mut external_items: Vec<_> = quote
        .external()
        .iter()
        .map(|(lang, key)| (lang.as_str().to_string(), key.as_str().to_string()))
        .collect();
    external_items.sort_by(|a, b| a.0.cmp(&b.0));
    push_items(lines, external_items);
}

fn push_markdown_section(lines: &mut Vec<Line<'static>>, quote: &Quote) {
    lines.push(Line::from(""));
    lines.push(Line::from("markdown:"));
    let mut markdown_items: Vec<_> = quote
        .markdown()
        .iter()
        .map(|(lang, key)| (lang.as_str().to_string(), key.as_str().to_string()))
        .collect();
    markdown_items.sort_by(|a, b| a.0.cmp(&b.0));
    push_items(lines, markdown_items);
}

fn push_image_section(lines: &mut Vec<Line<'static>>, quote: &Quote) {
    lines.push(Line::from(""));
    lines.push(Line::from("images:"));
    if quote.image().is_empty() {
        lines.push(Line::from("  <empty>"));
    } else {
        for (idx, key) in quote.image().iter().enumerate() {
            lines.push(Line::from(format!("  [{idx}] {}", key.as_str())));
        }
    }
}

fn push_items(lines: &mut Vec<Line<'static>>, items: Vec<(String, String)>) {
    if items.is_empty() {
        lines.push(Line::from("  <empty>"));
    } else {
        for (lang, value) in items {
            lines.push(Line::from(format!("  {lang}: {value}")));
        }
    }
}
