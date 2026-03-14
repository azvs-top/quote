use ratatui::layout::Rect;
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

pub(super) fn render(frame: &mut ratatui::Frame<'_>, area: Rect) {
    frame.render_widget(
        Paragraph::new(help_text())
            .block(Block::default().title("Help").borders(Borders::ALL))
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn help_text() -> Text<'static> {
    Text::from(vec![
        Line::from("Navigation"),
        Line::from("  j/k or ↓/↑    move selection"),
        Line::from("  J/K           jump last/first item in current page"),
        Line::from("  h/l or ←/→    prev/next page"),
        Line::from("  H/L           first/last page"),
        Line::from("  Enter         open selected item detail"),
        Line::from("  r             reload current page"),
        Line::from("  q             quit"),
        Line::from(""),
        Line::from("Command Mode"),
        Line::from("  :help         open this help page"),
        Line::from("  :list         back to list page"),
        Line::from("  :q or :quit   quit"),
        Line::from("  Esc           cancel command input"),
    ])
}
