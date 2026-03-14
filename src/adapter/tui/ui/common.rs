use crate::domain::entity::Quote;

pub(super) fn preview_inline(quote: &Quote) -> String {
    if let Some((_, text)) = quote.inline().iter().find(|(lang, _)| lang.as_str() == "en") {
        return text.clone();
    }
    if let Some((_, text)) = quote.inline().iter().find(|(lang, _)| lang.as_str() == "zh") {
        return text.clone();
    }
    if let Some((_, text)) = quote.inline().iter().next() {
        return text.clone();
    }
    "<no inline>".to_string()
}

pub(super) fn preview_inline_full(quote: &Quote) -> String {
    preview_inline(quote)
}
