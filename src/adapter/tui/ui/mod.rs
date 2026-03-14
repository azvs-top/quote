mod common;
mod detail;
mod help;
mod layout;
mod list;

use crate::adapter::tui::app::{Screen, TuiApp};
use ratatui::widgets::{Block, Borders, Paragraph};

pub(super) fn draw(frame: &mut ratatui::Frame<'_>, app: &TuiApp) {
    let parts = layout::split_root(frame.area());
    layout::render_title(frame, parts.title, app);

    // 帮助页是独立视图，打开后不渲染列表/详情双栏。
    if app.screen == Screen::Help {
        help::render(frame, parts.body);
        frame.render_widget(
            Paragraph::new(app.status_line()).block(Block::default().borders(Borders::TOP)),
            parts.status,
        );
        return;
    }

    if app.screen == Screen::Detail {
        detail::render_single(frame, parts.body, app);
        frame.render_widget(
            Paragraph::new(app.status_line()).block(Block::default().borders(Borders::TOP)),
            parts.status,
        );
        return;
    }

    let body = layout::split_body(parts.body);
    list::render(frame, body.list, app);
    detail::render_preview(frame, body.detail, app);
    frame.render_widget(
        Paragraph::new(app.status_line()).block(Block::default().borders(Borders::TOP)),
        parts.status,
    );
}
