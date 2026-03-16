use crate::adapter::tui::message::Msg;
use crate::adapter::tui::state::{GotoPage, TuiState};
use crate::application::quote::{QuotePort, QuoteQuery};
use crate::application::service::quote::{CountQuoteService, ListQuoteService};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use tokio::runtime::Handle;
use tokio::task::block_in_place;

pub(crate) struct Updater {
    quote_port: Arc<dyn QuotePort + Send + Sync>,
}

impl Updater {
    pub(crate) fn new(quote_port: Arc<dyn QuotePort + Send + Sync>) -> Self {
        Self { quote_port }
    }

    pub(crate) fn load_initial(&self, model: &Rc<RefCell<TuiState>>) {
        if let Err(err) = self.reload_full(model) {
            model.borrow_mut().set_error("load failed", &err);
        }
    }

    pub(crate) fn update(&self, model: &Rc<RefCell<TuiState>>, msg: Msg) -> anyhow::Result<bool> {
        let mut should_quit = false;
        match msg {
            Msg::Reload => {
                if let Err(err) = self.reload_full(model) {
                    model.borrow_mut().set_error("reload failed", &err);
                }
            }
            Msg::NextItem => model.borrow_mut().select_next(),
            Msg::PrevItem => model.borrow_mut().select_prev(),
            Msg::FirstItem => model.borrow_mut().select_first(),
            Msg::LastItem => model.borrow_mut().select_last(),
            Msg::NextPage => self.change_page_by(model, 1)?,
            Msg::PrevPage => self.change_page_by(model, -1)?,
            Msg::FirstPage => self.set_page(model, 1, "first page failed")?,
            Msg::LastPage => {
                let last = model.borrow().view.max_page();
                self.set_page(model, last, "last page failed")?;
            }
            Msg::OpenDetail => model.borrow_mut().open_detail(),
            Msg::BackToList => model.borrow_mut().back_to_list(),
            Msg::ToggleHelpLocale => model.borrow_mut().toggle_help_locale(),
            Msg::EnterCommand => model.borrow_mut().enter_command_mode(),
            Msg::AppendCommand(ch) => model.borrow_mut().append_command_char(ch),
            Msg::PopCommand => model.borrow_mut().pop_command_char(),
            Msg::CancelCommand => model.borrow_mut().cancel_command_mode(),
            Msg::ExecuteCommand => should_quit = self.execute_command(model)?,
            Msg::OpenGoto => model.borrow_mut().enter_goto_mode(),
            Msg::NextGoto => model.borrow_mut().goto_next(),
            Msg::PrevGoto => model.borrow_mut().goto_prev(),
            Msg::CancelGoto => model.borrow_mut().cancel_goto_mode(),
            Msg::ConfirmGoto => self.confirm_goto(model),
            Msg::ScrollDetailDown => model.borrow_mut().scroll_detail_down(),
            Msg::ScrollDetailUp => model.borrow_mut().scroll_detail_up(),
        }
        Ok(should_quit)
    }

    fn execute_command(&self, model: &Rc<RefCell<TuiState>>) -> anyhow::Result<bool> {
        let raw = model.borrow_mut().take_command();
        let command = raw.trim().strip_prefix(':').unwrap_or(raw.trim()).trim();
        let should_quit = match command {
            "" => {
                model.borrow_mut().restore_status_for_screen();
                false
            }
            "help" => {
                model.borrow_mut().open_help();
                false
            }
            "q" | "quit" => true,
            _ => {
                model.borrow_mut().view.status = format!("unknown command: :{command}");
                false
            }
        };
        Ok(should_quit)
    }

    fn confirm_goto(&self, model: &Rc<RefCell<TuiState>>) {
        let target = {
            let mut state = model.borrow_mut();
            let target = state.goto_target();
            state.overlay.goto.cancel();
            target
        };
        match target {
            GotoPage::List => model.borrow_mut().back_to_list(),
            GotoPage::Help => model.borrow_mut().open_help(),
        }
    }

    fn change_page_by(&self, model: &Rc<RefCell<TuiState>>, delta: i64) -> anyhow::Result<()> {
        let current = model.borrow().view.page;
        self.set_page(
            model,
            current + delta,
            if delta > 0 {
                "next page failed"
            } else {
                "prev page failed"
            },
        )
    }

    fn set_page(
        &self,
        model: &Rc<RefCell<TuiState>>,
        target: i64,
        err_prefix: &str,
    ) -> anyhow::Result<()> {
        let clamped = {
            let state = model.borrow();
            target.clamp(1, state.view.max_page())
        };
        {
            let mut state = model.borrow_mut();
            if clamped == state.view.page {
                return Ok(());
            }
            state.view.page = clamped;
        }
        if let Err(err) = self.reload_page(model) {
            model.borrow_mut().set_error(err_prefix, &err);
        }
        Ok(())
    }

    fn reload_full(&self, model: &Rc<RefCell<TuiState>>) -> anyhow::Result<()> {
        let total = self.block_on(async {
            let count_service = CountQuoteService::new(self.quote_port.as_ref());
            count_service.execute(QuoteQuery::builder().build()).await
        })?;
        {
            let mut state = model.borrow_mut();
            state.view.total = total;
            let max_page = state.view.max_page();
            if state.view.page > max_page {
                state.view.page = max_page;
            }
        }
        self.reload_page(model)
    }

    fn reload_page(&self, model: &Rc<RefCell<TuiState>>) -> anyhow::Result<()> {
        let (page, limit) = {
            let state = model.borrow();
            (state.view.page, state.view.limit)
        };
        let offset = (page - 1) * limit;
        let quotes = self.block_on(async {
            let query = QuoteQuery::builder()
                .with_limit(Some(limit))
                .with_offset(Some(offset))
                .build();
            let service = ListQuoteService::new(self.quote_port.as_ref());
            service.execute(query).await
        })?;
        {
            let mut state = model.borrow_mut();
            state.view.quotes = quotes;
            state.sync_after_reload();
            state.restore_status_for_screen();
        }
        Ok(())
    }

    fn block_on<F, T>(&self, future: F) -> T
    where
        F: std::future::Future<Output = T>,
    {
        block_in_place(|| Handle::current().block_on(future))
    }
}
