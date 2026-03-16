use crate::adapter::tui::app::TuiApp;
use crate::adapter::tui::ui::common::preview_inline;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub(super) fn render(frame: &mut ratatui::Frame<'_>, area: Rect, app: &TuiApp) {
    // 列表边界和高亮前缀 ("> ").
    let row_budget = area.width.saturating_sub(4) as usize;
    let id_width = app
        .quotes
        .iter()
        .map(|q| q.id().to_string().len())
        .max()
        .unwrap_or(1);

    let mut list_state = ListState::default();
    if !app.quotes.is_empty() {
        list_state.select(Some(app.selected));
    }
    let items: Vec<ListItem<'_>> = if app.quotes.is_empty() {
        vec![ListItem::new("no data")]
    } else {
        app.quotes
            .iter()
            .map(|q| ListItem::new(build_row_text(q, row_budget, id_width)))
            .collect()
    };
    let list = List::new(items)
        .block(Block::default().title("Quotes").borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");
    frame.render_stateful_widget(list, area, &mut list_state);
}

fn build_row_text(
    quote: &crate::domain::entity::Quote,
    row_budget: usize,
    id_width: usize,
) -> String {
    let prefix = format!("{:>width$} ", quote.id(), width = id_width);
    if row_budget <= UnicodeWidthStr::width(prefix.as_str()) {
        return trim_to_width(&prefix, row_budget);
    }

    let rest_budget = row_budget - UnicodeWidthStr::width(prefix.as_str());
    let inline = preview_inline(quote);
    format!("{}{}", prefix, trim_to_width(&inline, rest_budget))
}

fn trim_to_width(input: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }
    if UnicodeWidthStr::width(input) <= max_width {
        return input.to_string();
    }
    if max_width <= 1 {
        return "…".to_string();
    }

    let mut out = String::new();
    let mut used = 0usize;
    let limit = max_width - 1; // reserve for ellipsis
    for ch in input.chars() {
        let w = ch.width().unwrap_or(0);
        if used + w > limit {
            break;
        }
        out.push(ch);
        used += w;
    }
    out.push('…');
    out
}
