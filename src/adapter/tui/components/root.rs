use crate::adapter::tui::message::Msg;
use crate::adapter::tui::state::TuiState;
use crate::adapter::tui::ui;
use std::cell::RefCell;
use std::rc::Rc;
use tuirealm::command::{Cmd, CmdResult};
use tuirealm::ratatui::layout::Rect;
use tuirealm::{
    AttrValue, Attribute, Component, Event, Frame, MockComponent, NoUserEvent, Props, State,
};

pub(crate) struct Root {
    props: Props,
    state: Rc<RefCell<TuiState>>,
}

impl Root {
    pub(crate) fn new(state: Rc<RefCell<TuiState>>) -> Self {
        Self {
            props: Props::default(),
            state,
        }
    }
}

impl MockComponent for Root {
    fn view(&mut self, frame: &mut Frame, area: Rect) {
        let state = self.state.borrow();
        ui::draw(frame, area, &state);
    }

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

impl Component<Msg, NoUserEvent> for Root {
    fn on(&mut self, _: Event<NoUserEvent>) -> Option<Msg> {
        None
    }
}
