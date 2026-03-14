use crate::adapter::tui::app::TuiApp;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

pub(super) struct RootParts {
    pub(super) title: Rect,
    pub(super) body: Rect,
    pub(super) status: Rect,
}

pub(super) struct BodyParts {
    pub(super) list: Rect,
    pub(super) detail: Rect,
}

pub(super) fn split_root(area: Rect) -> RootParts {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0), Constraint::Length(2)])
        .split(area);
    RootParts {
        title: root[0],
        body: root[1],
        status: root[2],
    }
}

pub(super) fn split_body(area: Rect) -> BodyParts {
    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
        .split(area);
    BodyParts {
        list: body[0],
        detail: body[1],
    }
}

pub(super) fn render_title(frame: &mut ratatui::Frame<'_>, area: Rect, app: &TuiApp) {
    let title = Line::from(vec![
        Span::styled("quote-tui", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::styled(
            format!(
                "total={} page={} limit={} items={}",
                app.total,
                app.page,
                app.limit,
                app.quotes.len()
            ),
            Style::default().fg(Color::DarkGray),
        ),
    ]);
    frame.render_widget(Paragraph::new(title), area);
}
