use super::{RouterEvent, RouterState};
use std::fmt;
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ProtocolError {
    InvalidIteration {
        current: usize,
        max: usize,
    },
    InvalidEvent {
        state: RouterState,
        event: RouterEvent,
    },
}
impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidIteration { current, max } => {
                write!(f, "invalid router iteration {current}/{max}")
            }
            Self::InvalidEvent { state, event } => {
                write!(f, "router event {event:?} is invalid in state {state:?}")
            }
        }
    }
}
