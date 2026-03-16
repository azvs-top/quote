use crate::adapter::tui::state::TuiState;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState};

pub(crate) fn render(frame: &mut ratatui::Frame<'_>, area: Rect, state: &TuiState) {
    let popup = centered_popup(area, 46, 8);
    frame.render_widget(Clear, popup);

    let entries = state.goto_entries();
    let items: Vec<ListItem<'_>> = entries
        .iter()
        .map(|item| ListItem::new(item.as_str()))
        .collect();
    let mut list_state = ListState::default();
    list_state.select(Some(state.overlay.goto.selected()));

    let list = List::new(items)
        .block(Block::default().title("Goto").borders(Borders::ALL))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");
    frame.render_stateful_widget(list, popup, &mut list_state);
}

fn centered_popup(area: Rect, width: u16, height: u16) -> Rect {
    let w = width.min(area.width.saturating_sub(2)).max(20);
    let h = height.min(area.height.saturating_sub(2)).max(5);
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(area.height.saturating_sub(h) / 2),
            Constraint::Length(h),
            Constraint::Min(0),
        ])
        .split(area);
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(area.width.saturating_sub(w) / 2),
            Constraint::Length(w),
            Constraint::Min(0),
        ])
        .split(vertical[1]);
    horizontal[1]
}
