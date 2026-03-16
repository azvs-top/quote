use crate::domain::entity::Quote;

const READY_HINT: &str = "ready: j/k move, n/p page, l enter, g goto, :help commands";
const HELP_HINT: &str = "help page opened";
const DETAIL_HINT: &str = "detail: j/k scroll, h back, g goto, :quit";
const GOTO_HINT: &str = "goto: j/k choose, Enter jump, Esc cancel";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Screen {
    List,
    Detail,
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HelpLocale {
    En,
    Zh,
}

impl HelpLocale {
    pub(crate) fn toggle(self) -> Self {
        match self {
            Self::En => Self::Zh,
            Self::Zh => Self::En,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GotoPage {
    List,
    Help,
}

impl GotoPage {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::List => "List",
            Self::Help => "Help",
        }
    }

    pub(crate) fn screen(self) -> Screen {
        match self {
            Self::List => Screen::List,
            Self::Help => Screen::Help,
        }
    }
}

pub(crate) const GOTO_PAGES: [GotoPage; 2] = [GotoPage::List, GotoPage::Help];

#[derive(Default)]
pub(crate) struct CommandState {
    active: bool,
    input: String,
}

impl CommandState {
    pub(crate) fn is_active(&self) -> bool {
        self.active
    }

    pub(crate) fn status_line(&self) -> Option<String> {
        self.active.then(|| self.input.clone())
    }

    pub(crate) fn enter(&mut self) {
        self.active = true;
        self.input = ":".to_string();
    }

    pub(crate) fn append_char(&mut self, ch: char) {
        if self.active {
            self.input.push(ch);
        }
    }

    pub(crate) fn pop_char(&mut self) {
        if self.active && self.input.len() > 1 {
            self.input.pop();
        }
    }

    pub(crate) fn cancel(&mut self) {
        self.active = false;
        self.input.clear();
    }

    pub(crate) fn take(&mut self) -> String {
        let raw = self.input.trim().to_string();
        self.cancel();
        raw
    }
}

#[derive(Default)]
pub(crate) struct GotoState {
    active: bool,
    selected: usize,
}

impl GotoState {
    pub(crate) fn is_active(&self) -> bool {
        self.active
    }

    pub(crate) fn selected(&self) -> usize {
        self.selected
    }

    pub(crate) fn enter(&mut self) {
        self.active = true;
        self.selected = 0;
    }

    pub(crate) fn cancel(&mut self) {
        self.active = false;
    }

    pub(crate) fn next(&mut self) {
        if self.active {
            self.selected = (self.selected + 1).min(GOTO_PAGES.len() - 1);
        }
    }

    pub(crate) fn prev(&mut self) {
        if self.active {
            self.selected = self.selected.saturating_sub(1);
        }
    }

    pub(crate) fn target(&self) -> GotoPage {
        GOTO_PAGES[self.selected]
    }
}

#[derive(Default)]
pub(crate) struct OverlayState {
    pub(crate) command: CommandState,
    pub(crate) goto: GotoState,
}

pub(crate) struct ViewState {
    pub(crate) quotes: Vec<Quote>,
    pub(crate) page: i64,
    pub(crate) limit: i64,
    pub(crate) selected: usize,
    pub(crate) total: i64,
    pub(crate) status: String,
    pub(crate) screen: Screen,
    pub(crate) help_locale: HelpLocale,
    pub(crate) detail_scroll: u16,
    pub(crate) detail_scroll_cap: u16,
}

impl Default for ViewState {
    fn default() -> Self {
        Self {
            quotes: Vec::new(),
            page: 1,
            limit: 10,
            selected: 0,
            total: 0,
            status: READY_HINT.to_string(),
            screen: Screen::List,
            help_locale: HelpLocale::En,
            detail_scroll: 0,
            detail_scroll_cap: 0,
        }
    }
}

#[derive(Default)]
pub(crate) struct TuiState {
    pub(crate) view: ViewState,
    pub(crate) overlay: OverlayState,
}

impl ViewState {
    pub(crate) fn max_page(&self) -> i64 {
        if self.total <= 0 {
            1
        } else {
            ((self.total - 1) / self.limit) + 1
        }
    }
}

impl TuiState {
    pub(crate) fn selected_quote(&self) -> Option<&Quote> {
        self.view.quotes.get(self.view.selected)
    }

    pub(crate) fn sync_after_reload(&mut self) {
        if self.view.selected >= self.view.quotes.len() {
            self.view.selected = self.view.quotes.len().saturating_sub(1);
        }
        if self.view.quotes.is_empty() {
            self.view.selected = 0;
        }
        self.rebuild_detail_scroll_cap();
        if self.view.detail_scroll > self.view.detail_scroll_cap {
            self.view.detail_scroll = self.view.detail_scroll_cap;
        }
    }

    pub(crate) fn status_line(&self) -> String {
        if let Some(line) = self.overlay.command.status_line() {
            line
        } else {
            self.view.status.clone()
        }
    }

    pub(crate) fn restore_status_for_screen(&mut self) {
        self.view.status = match self.view.screen {
            Screen::List => READY_HINT.to_string(),
            Screen::Detail => DETAIL_HINT.to_string(),
            Screen::Help => HELP_HINT.to_string(),
        };
    }

    pub(crate) fn set_error(&mut self, prefix: &str, err: &anyhow::Error) {
        self.view.status = format!("{prefix}: {err}");
    }

    pub(crate) fn select_next(&mut self) {
        if !self.view.quotes.is_empty() {
            self.view.selected = (self.view.selected + 1).min(self.view.quotes.len() - 1);
        }
    }

    pub(crate) fn select_prev(&mut self) {
        if !self.view.quotes.is_empty() {
            self.view.selected = self.view.selected.saturating_sub(1);
        }
    }

    pub(crate) fn select_first(&mut self) {
        self.view.selected = 0;
    }

    pub(crate) fn select_last(&mut self) {
        if !self.view.quotes.is_empty() {
            self.view.selected = self.view.quotes.len() - 1;
        }
    }

    pub(crate) fn open_detail(&mut self) {
        if self.selected_quote().is_some() {
            self.view.screen = Screen::Detail;
            self.view.detail_scroll = 0;
            self.rebuild_detail_scroll_cap();
            self.restore_status_for_screen();
        }
    }

    pub(crate) fn open_help(&mut self) {
        self.view.screen = Screen::Help;
        self.view.detail_scroll = 0;
        self.restore_status_for_screen();
    }

    pub(crate) fn toggle_help_locale(&mut self) {
        self.view.help_locale = self.view.help_locale.toggle();
    }

    pub(crate) fn back_to_list(&mut self) {
        self.view.screen = Screen::List;
        self.view.detail_scroll = 0;
        self.restore_status_for_screen();
    }

    pub(crate) fn scroll_detail_down(&mut self) {
        self.view.detail_scroll = self
            .view
            .detail_scroll
            .saturating_add(1)
            .min(self.view.detail_scroll_cap);
    }

    pub(crate) fn scroll_detail_up(&mut self) {
        self.view.detail_scroll = self.view.detail_scroll.saturating_sub(1);
    }

    pub(crate) fn enter_command_mode(&mut self) {
        self.overlay.command.enter();
    }

    pub(crate) fn append_command_char(&mut self, ch: char) {
        self.overlay.command.append_char(ch);
    }

    pub(crate) fn pop_command_char(&mut self) {
        self.overlay.command.pop_char();
    }

    pub(crate) fn cancel_command_mode(&mut self) {
        self.overlay.command.cancel();
        self.restore_status_for_screen();
    }

    pub(crate) fn take_command(&mut self) -> String {
        self.overlay.command.take()
    }

    pub(crate) fn enter_goto_mode(&mut self) {
        self.overlay.goto.enter();
        self.view.status = GOTO_HINT.to_string();
    }

    pub(crate) fn cancel_goto_mode(&mut self) {
        if self.overlay.goto.is_active() {
            self.overlay.goto.cancel();
            self.restore_status_for_screen();
        }
    }

    pub(crate) fn goto_next(&mut self) {
        self.overlay.goto.next();
    }

    pub(crate) fn goto_prev(&mut self) {
        self.overlay.goto.prev();
    }

    pub(crate) fn goto_target(&self) -> GotoPage {
        self.overlay.goto.target()
    }

    pub(crate) fn goto_entries(&self) -> Vec<String> {
        GOTO_PAGES
            .iter()
            .map(|page| {
                let mut label = page.label().to_string();
                if page.screen() == self.view.screen {
                    label.push_str(" (current)");
                }
                label
            })
            .collect()
    }

    fn rebuild_detail_scroll_cap(&mut self) {
        let Some(q) = self.selected_quote() else {
            self.view.detail_scroll_cap = 0;
            return;
        };
        let mut lines = 5usize;
        lines += q.inline().len().max(1) + 2;
        lines += q.external().len().max(1) + 2;
        lines += q.markdown().len().max(1) + 2;
        lines += q.image().len().max(1) + 1;
        self.view.detail_scroll_cap = lines.min(u16::MAX as usize) as u16;
    }
}
