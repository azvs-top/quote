use crate::adapter::tui::state::{HelpLocale, Screen, TuiState};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

pub(crate) struct RootParts {
    pub(crate) title: Rect,
    pub(crate) body: Rect,
    pub(crate) status: Rect,
}

pub(crate) struct BodyParts {
    pub(crate) list: Rect,
    pub(crate) detail: Rect,
}

pub(crate) fn split_root(area: Rect) -> RootParts {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(2),
        ])
        .split(area);
    RootParts {
        title: root[0],
        body: root[1],
        status: root[2],
    }
}

pub(crate) fn split_body(area: Rect) -> BodyParts {
    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
        .split(area);
    BodyParts {
        list: body[0],
        detail: body[1],
    }
}

pub(crate) fn render_title(frame: &mut ratatui::Frame<'_>, area: Rect, state: &TuiState) {
    if state.view.screen == Screen::Help {
        let locale = match state.view.help_locale {
            HelpLocale::En => "EN",
            HelpLocale::Zh => "ZH",
        };
        let title = Line::from(vec![
            Span::styled("help", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("  "),
            Span::styled(
                format!("press t to switch language  current={locale}"),
                Style::default().fg(Color::DarkGray),
            ),
        ]);
        frame.render_widget(Paragraph::new(title), area);
        return;
    }

    if state.view.screen == Screen::Detail {
        let title = match state.selected_quote() {
            Some(quote) => Line::from(vec![
                Span::styled("detail", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw("  "),
                Span::styled(
                    format!(
                        "inline: {} external: {} markdown: {} images: {}",
                        quote.inline().len(),
                        quote.external().len(),
                        quote.markdown().len(),
                        quote.image().len()
                    ),
                    Style::default().fg(Color::DarkGray),
                ),
            ]),
            None => Line::from(vec![
                Span::styled("detail", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw("  "),
                Span::styled("no quote selected", Style::default().fg(Color::DarkGray)),
            ]),
        };
        frame.render_widget(Paragraph::new(title), area);
        return;
    }

    let title = Line::from(vec![
        Span::styled("quote-tui", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::styled(
            format!(
                "total={} page={} limit={} items={}",
                state.view.total,
                state.view.page,
                state.view.limit,
                state.view.quotes.len()
            ),
            Style::default().fg(Color::DarkGray),
        ),
    ]);
    frame.render_widget(Paragraph::new(title), area);
}
