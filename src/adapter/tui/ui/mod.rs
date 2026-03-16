mod common;
mod detail;
mod help;
mod jump;
mod layout;
mod list;

use crate::adapter::tui::state::{Screen, TuiState};
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Paragraph};

pub(crate) fn draw(frame: &mut ratatui::Frame<'_>, area: Rect, state: &TuiState) {
    let parts = layout::split_root(area);
    layout::render_title(frame, parts.title, state);

    match state.view.screen {
        Screen::Help => help::render(frame, parts.body, state),
        Screen::Detail => detail::render_single(frame, parts.body, state),
        Screen::List => {
            let body = layout::split_body(parts.body);
            list::render(frame, body.list, state);
            detail::render_preview(frame, body.detail, state);
        }
    }

    frame.render_widget(
        Paragraph::new(state.status_line()).block(Block::default().borders(Borders::TOP)),
        parts.status,
    );

    if state.overlay.goto.is_active() {
        jump::render(frame, parts.body, state);
    }
}
