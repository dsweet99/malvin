#[path = "router_machine/domain.rs"]
mod domain;
#[path = "router_machine/error.rs"]
mod error;
#[path = "router_machine/protocol.rs"]
mod protocol;
#[path = "router_machine/transition.rs"]
mod transition;
pub(crate) use domain::{Completion, Iteration, RunOutcome};
pub(crate) use error::ProtocolError;
pub(crate) use protocol::{Effect, RouterEvent, RouterState, Transition};
pub(crate) use transition::transition;
#[cfg(test)]
#[path = "router_machine/tests.rs"]
mod tests;
