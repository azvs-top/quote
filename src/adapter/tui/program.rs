use crate::adapter::tui::components::{Controller, Id, Root};
use crate::adapter::tui::message::Msg;
use crate::adapter::tui::state::TuiState;
use crate::adapter::tui::update::Updater;
use crate::application::quote::QuotePort;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;
use tuirealm::application::PollStrategy;
use tuirealm::event::NoUserEvent;
use tuirealm::terminal::{CrosstermTerminalAdapter, TerminalBridge};
use tuirealm::{Application, EventListenerCfg};

pub(crate) struct Program {
    app: Application<Id, Msg, NoUserEvent>,
    terminal: TerminalBridge<CrosstermTerminalAdapter>,
    model: Rc<RefCell<TuiState>>,
    updater: Updater,
    pub(crate) quit: bool,
    pub(crate) redraw: bool,
}

impl Program {
    pub(crate) fn new(quote_port: Arc<dyn QuotePort + Send + Sync>) -> anyhow::Result<Self> {
        let model = Rc::new(RefCell::new(TuiState::default()));
        let mut app: Application<Id, Msg, NoUserEvent> = Application::init(
            EventListenerCfg::default()
                .crossterm_input_listener(Duration::from_millis(20), 4)
                .poll_timeout(Duration::from_millis(20)),
        );
        app.mount(Id::Root, Box::new(Root::new(model.clone())), Vec::new())?;
        app.mount(
            Id::Controller,
            Box::new(Controller::new(model.clone())),
            Vec::new(),
        )?;
        app.active(&Id::Controller)?;

        Ok(Self {
            app,
            terminal: TerminalBridge::init_crossterm()?,
            model,
            updater: Updater::new(quote_port),
            quit: false,
            redraw: true,
        })
    }

    pub(crate) fn load_initial(&self) {
        self.updater.load_initial(&self.model);
    }

    pub(crate) fn tick(&mut self) -> anyhow::Result<()> {
        let messages = self.app.tick(PollStrategy::Once)?;
        if !messages.is_empty() {
            self.redraw = true;
        }
        for msg in messages {
            if self.updater.update(&self.model, msg)? {
                self.quit = true;
            }
        }
        Ok(())
    }

    pub(crate) fn draw(&mut self) -> anyhow::Result<()> {
        self.terminal
            .draw(|frame| self.app.view(&Id::Root, frame, frame.area()))?;
        self.redraw = false;
        Ok(())
    }

    pub(crate) fn restore(&mut self) -> anyhow::Result<()> {
        self.terminal.restore()?;
        self.terminal.clear_screen()?;
        Ok(())
    }
}
