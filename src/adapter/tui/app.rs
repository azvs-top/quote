use crate::application::quote::{QuotePort, QuoteQuery};
use crate::application::service::quote::{CountQuoteService, ListQuoteService};
use crate::domain::entity::Quote;
use std::sync::Arc;

const READY_HINT: &str = "ready: q quit, :help commands";
const HELP_HINT: &str = "help page opened (use :list to go back)";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Screen {
    List,
    Detail,
    Help,
}

pub(super) struct TuiApp {
    quote_port: Arc<dyn QuotePort + Send + Sync>,
    pub(super) quotes: Vec<Quote>,
    pub(super) page: i64,
    pub(super) limit: i64,
    pub(super) selected: usize,
    pub(super) total: i64,
    pub(super) status: String,
    pub(super) screen: Screen,
    pub(super) detail_scroll: u16,
    detail_scroll_cap: u16,
    pub(super) command_mode: bool,   // true 时键盘输入进入 ":" 命令行
    pub(super) command_input: String,
    pub(super) should_quit: bool,
}

impl TuiApp {
    pub(super) fn new(quote_port: Arc<dyn QuotePort + Send + Sync>) -> Self {
        Self {
            quote_port,
            quotes: Vec::new(),
            page: 1,
            limit: 10,
            selected: 0,
            total: 0,
            status: READY_HINT.to_string(),
            screen: Screen::List,
            detail_scroll: 0,
            detail_scroll_cap: 0,
            command_mode: false,
            command_input: String::new(),
            should_quit: false,
        }
    }

    pub(super) async fn reload_full(&mut self) -> anyhow::Result<()> {
        // 先取总数再计算可用最大页，避免翻页后越界。
        let count_service = CountQuoteService::new(self.quote_port.as_ref());
        self.total = count_service.execute(QuoteQuery::builder().build()).await?;
        let max_page = self.max_page();
        if self.page > max_page {
            self.page = max_page;
        }
        self.reload_page().await
    }

    pub(super) async fn reload_page(&mut self) -> anyhow::Result<()> {
        let offset = (self.page - 1) * self.limit;
        let query = QuoteQuery::builder()
            .with_limit(Some(self.limit))
            .with_offset(Some(offset))
            .build();
        let service = ListQuoteService::new(self.quote_port.as_ref());
        self.quotes = service.execute(query).await?;

        if self.selected >= self.quotes.len() {
            self.selected = self.quotes.len().saturating_sub(1);
        }
        if self.quotes.is_empty() {
            self.selected = 0;
        }
        self.rebuild_detail_scroll_cap();
        if self.detail_scroll > self.detail_scroll_cap {
            self.detail_scroll = self.detail_scroll_cap;
        }
        Ok(())
    }

    pub(super) fn select_next(&mut self) {
        if self.quotes.is_empty() {
            return;
        }
        self.selected = (self.selected + 1).min(self.quotes.len() - 1);
    }

    pub(super) fn select_prev(&mut self) {
        if self.quotes.is_empty() {
            return;
        }
        self.selected = self.selected.saturating_sub(1);
    }

    pub(super) fn selected_quote(&self) -> Option<&Quote> {
        self.quotes.get(self.selected)
    }

    pub(super) fn select_first(&mut self) {
        if self.quotes.is_empty() {
            self.selected = 0;
            return;
        }
        self.selected = 0;
    }

    pub(super) fn select_last(&mut self) {
        if self.quotes.is_empty() {
            self.selected = 0;
            return;
        }
        self.selected = self.quotes.len() - 1;
    }

    pub(super) fn max_page(&self) -> i64 {
        if self.total <= 0 {
            return 1;
        }
        ((self.total - 1) / self.limit) + 1
    }

    pub(super) fn reset_status(&mut self) {
        self.status = READY_HINT.to_string();
    }

    pub(super) fn enter_command_mode(&mut self) {
        self.command_mode = true;
        self.command_input = ":".to_string();
    }

    pub(super) fn append_command_char(&mut self, ch: char) {
        if !self.command_mode {
            return;
        }
        self.command_input.push(ch);
    }

    pub(super) fn pop_command_char(&mut self) {
        if !self.command_mode {
            return;
        }
        if self.command_input.len() > 1 {
            self.command_input.pop();
        }
    }

    pub(super) fn cancel_command_mode(&mut self) {
        self.command_mode = false;
        self.command_input.clear();
        self.reset_status();
    }

    pub(super) fn execute_command(&mut self) {
        // 统一命令入口，便于后续扩展 :set / :open 等子命令。
        let raw = self.command_input.trim();
        let cmd = raw.strip_prefix(':').unwrap_or(raw).trim();
        match cmd {
            "" => self.reset_status(),
            "help" => {
                self.screen = Screen::Help;
                self.status = HELP_HINT.to_string();
            }
            "list" => {
                self.back_to_list();
            }
            "q" | "quit" => {
                self.should_quit = true;
            }
            _ => {
                self.status = format!("unknown command: :{cmd}");
            }
        }
        self.command_mode = false;
        self.command_input.clear();
    }

    pub(super) fn status_line(&self) -> String {
        if self.command_mode {
            return self.command_input.clone();
        }
        self.status.clone()
    }

    pub(super) fn open_detail(&mut self) {
        if self.selected_quote().is_some() {
            self.screen = Screen::Detail;
            self.detail_scroll = 0;
            self.rebuild_detail_scroll_cap();
            self.status = "detail: :list back, :help commands, q quit".to_string();
        }
    }

    pub(super) fn back_to_list(&mut self) {
        self.screen = Screen::List;
        self.detail_scroll = 0;
        self.reset_status();
    }

    pub(super) fn scroll_detail_down(&mut self) {
        self.detail_scroll = self.detail_scroll.saturating_add(1).min(self.detail_scroll_cap);
    }

    pub(super) fn scroll_detail_up(&mut self) {
        self.detail_scroll = self.detail_scroll.saturating_sub(1);
    }

    fn rebuild_detail_scroll_cap(&mut self) {
        let Some(q) = self.selected_quote() else {
            self.detail_scroll_cap = 0;
            return;
        };
        // Rough upper bound by rendered line count of detail page.
        let mut lines = 5usize; // id/remark/header blanks
        lines += q.inline().len().max(1) + 2;
        lines += q.external().len().max(1) + 2;
        lines += q.markdown().len().max(1) + 2;
        lines += q.image().len().max(1) + 1;
        self.detail_scroll_cap = lines.min(u16::MAX as usize) as u16;
    }
}
