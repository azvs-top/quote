use crate::adapter::tui::message::Msg;
use crate::adapter::tui::state::{Screen, TuiState};
use std::cell::RefCell;
use std::rc::Rc;
use tuirealm::command::{Cmd, CmdResult};
use tuirealm::event::{Key, KeyEvent};
use tuirealm::ratatui::layout::Rect;
use tuirealm::{
    AttrValue, Attribute, Component, Event, Frame, MockComponent, NoUserEvent, Props, State,
};

pub(crate) struct Controller {
    props: Props,
    state: Rc<RefCell<TuiState>>,
}

impl Controller {
    pub(crate) fn new(state: Rc<RefCell<TuiState>>) -> Self {
        Self {
            props: Props::default(),
            state,
        }
    }
}

impl MockComponent for Controller {
    fn view(&mut self, _: &mut Frame, _: Rect) {}

    fn query(&self, attr: Attribute) -> Option<AttrValue> {
        self.props.get(attr)
    }

    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        self.props.set(attr, value);
    }

    fn state(&self) -> State {
        State::None
    }

    fn perform(&mut self, _: Cmd) -> CmdResult {
        CmdResult::None
    }
}

impl Component<Msg, NoUserEvent> for Controller {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let snapshot = {
            let state = self.state.borrow();
            (
                state.overlay.command.is_active(),
                state.overlay.goto.is_active(),
                state.view.screen,
            )
        };

        match snapshot {
            (true, _, _) => handle_command_mode(ev),
            (false, true, _) => handle_goto_mode(ev),
            (false, false, Screen::Detail) => handle_detail_mode(ev),
            (false, false, Screen::Help) => handle_help_mode(ev),
            (false, false, Screen::List) => handle_list_mode(ev),
        }
    }
}

fn handle_command_mode(ev: Event<NoUserEvent>) -> Option<Msg> {
    match ev {
        Event::Keyboard(KeyEvent {
            code: Key::Enter, ..
        }) => Some(Msg::ExecuteCommand),
        Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => Some(Msg::CancelCommand),
        Event::Keyboard(KeyEvent {
            code: Key::Backspace,
            ..
        }) => Some(Msg::PopCommand),
        Event::Keyboard(KeyEvent {
            code: Key::Char(ch),
            ..
        }) => Some(Msg::AppendCommand(ch)),
        _ => None,
    }
}

fn handle_goto_mode(ev: Event<NoUserEvent>) -> Option<Msg> {
    match ev {
        Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => Some(Msg::CancelGoto),
        Event::Keyboard(KeyEvent {
            code: Key::Char('g'),
            ..
        }) => Some(Msg::CancelGoto),
        Event::Keyboard(KeyEvent {
            code: Key::Enter, ..
        }) => Some(Msg::ConfirmGoto),
        Event::Keyboard(KeyEvent {
            code: Key::Down, ..
        })
        | Event::Keyboard(KeyEvent {
            code: Key::Char('j'),
            ..
        }) => Some(Msg::NextGoto),
        Event::Keyboard(KeyEvent { code: Key::Up, .. })
        | Event::Keyboard(KeyEvent {
            code: Key::Char('k'),
            ..
        }) => Some(Msg::PrevGoto),
        _ => None,
    }
}

fn handle_detail_mode(ev: Event<NoUserEvent>) -> Option<Msg> {
    match ev {
        Event::Keyboard(KeyEvent {
            code: Key::Char(':'),
            ..
        }) => Some(Msg::EnterCommand),
        Event::Keyboard(KeyEvent {
            code: Key::Char('g'),
            ..
        }) => Some(Msg::OpenGoto),
        Event::Keyboard(KeyEvent {
            code: Key::Char('h'),
            ..
        }) => Some(Msg::BackToList),
        Event::Keyboard(KeyEvent {
            code: Key::Down, ..
        })
        | Event::Keyboard(KeyEvent {
            code: Key::Char('j'),
            ..
        }) => Some(Msg::ScrollDetailDown),
        Event::Keyboard(KeyEvent { code: Key::Up, .. })
        | Event::Keyboard(KeyEvent {
            code: Key::Char('k'),
            ..
        }) => Some(Msg::ScrollDetailUp),
        _ => None,
    }
}

fn handle_help_mode(ev: Event<NoUserEvent>) -> Option<Msg> {
    match ev {
        Event::Keyboard(KeyEvent {
            code: Key::Char(':'),
            ..
        }) => Some(Msg::EnterCommand),
        Event::Keyboard(KeyEvent {
            code: Key::Char('g'),
            ..
        }) => Some(Msg::OpenGoto),
        Event::Keyboard(KeyEvent {
            code: Key::Char('h'),
            ..
        }) => Some(Msg::BackToList),
        Event::Keyboard(KeyEvent {
            code: Key::Char('t'),
            ..
        }) => Some(Msg::ToggleHelpLocale),
        _ => None,
    }
}

fn handle_list_mode(ev: Event<NoUserEvent>) -> Option<Msg> {
    match ev {
        Event::Keyboard(KeyEvent {
            code: Key::Char(':'),
            ..
        }) => Some(Msg::EnterCommand),
        Event::Keyboard(KeyEvent {
            code: Key::Char('g'),
            ..
        }) => Some(Msg::OpenGoto),
        Event::Keyboard(KeyEvent {
            code: Key::Char('r'),
            ..
        }) => Some(Msg::Reload),
        Event::Keyboard(KeyEvent {
            code: Key::Down, ..
        })
        | Event::Keyboard(KeyEvent {
            code: Key::Char('j'),
            ..
        }) => Some(Msg::NextItem),
        Event::Keyboard(KeyEvent { code: Key::Up, .. })
        | Event::Keyboard(KeyEvent {
            code: Key::Char('k'),
            ..
        }) => Some(Msg::PrevItem),
        Event::Keyboard(KeyEvent {
            code: Key::Char('J'),
            ..
        }) => Some(Msg::LastItem),
        Event::Keyboard(KeyEvent {
            code: Key::Char('K'),
            ..
        }) => Some(Msg::FirstItem),
        Event::Keyboard(KeyEvent {
            code: Key::Right, ..
        })
        | Event::Keyboard(KeyEvent {
            code: Key::Char('n'),
            ..
        }) => Some(Msg::NextPage),
        Event::Keyboard(KeyEvent {
            code: Key::Left, ..
        })
        | Event::Keyboard(KeyEvent {
            code: Key::Char('p'),
            ..
        }) => Some(Msg::PrevPage),
        Event::Keyboard(KeyEvent {
            code: Key::Char('H'),
            ..
        }) => Some(Msg::FirstPage),
        Event::Keyboard(KeyEvent {
            code: Key::Char('L'),
            ..
        }) => Some(Msg::LastPage),
        Event::Keyboard(KeyEvent {
            code: Key::Char('l'),
            ..
        })
        | Event::Keyboard(KeyEvent {
            code: Key::Enter, ..
        }) => Some(Msg::OpenDetail),
        _ => None,
    }
}
