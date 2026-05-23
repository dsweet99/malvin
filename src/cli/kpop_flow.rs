//! KPOP subcommand: artifacts, prompt assembly, and ACP dispatch.

pub use crate::KpopTurnPrompts;

#[path = "kpop_flow_a.rs"]
mod kpop_flow_a;
#[path = "kpop_flow_b.rs"]
mod kpop_flow_b;

pub use kpop_flow_a::*;
pub use kpop_flow_b::*;
