use crate::adapter::tui::state::{HelpLocale, TuiState};
use ratatui::layout::Rect;
use ratatui::text::Text;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

const HELP_EN: &str = include_str!("../assets/help_en.txt");
const HELP_ZH: &str = include_str!("../assets/help_zh.txt");

pub(crate) fn render(frame: &mut ratatui::Frame<'_>, area: Rect, state: &TuiState) {
    frame.render_widget(
        Paragraph::new(help_text(state.view.help_locale))
            .block(Block::default().title("Help").borders(Borders::ALL))
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn help_text(locale: HelpLocale) -> Text<'static> {
    match locale {
        HelpLocale::En => Text::from(HELP_EN),
        HelpLocale::Zh => Text::from(HELP_ZH),
    }
}
