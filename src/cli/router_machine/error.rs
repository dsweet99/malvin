use super::{InvalidIteration, RouterEvent, RouterState};
use std::fmt;
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ProtocolError {
    InvalidIteration(InvalidIteration),
    InvalidEvent {
        state: RouterState,
        event: RouterEvent,
    },
}
impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidIteration(InvalidIteration { current, max }) => {
                write!(f, "invalid router iteration {current}/{max}")
            }
            Self::InvalidEvent { state, event } => {
                write!(f, "router event {event:?} is invalid in state {state:?}")
            }
        }
    }
}
impl From<InvalidIteration> for ProtocolError {
    fn from(invalid: InvalidIteration) -> Self {
        Self::InvalidIteration(invalid)
    }
}
