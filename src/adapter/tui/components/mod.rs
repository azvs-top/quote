mod controller;
mod root;

pub(crate) use controller::Controller;
pub(crate) use root::Root;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Id {
    Root,
    Controller,
}
