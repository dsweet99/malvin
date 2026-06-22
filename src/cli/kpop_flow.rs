//! `KPop` subcommand: artifacts, prompt assembly, and ACP dispatch.

#[path = "kpop_flow_a.rs"]
mod kpop_flow_a;
#[path = "kpop_flow_b.rs"]
mod kpop_flow_b;
#[path = "kpop_flow_run_loop.rs"]
pub(crate) mod kpop_flow_run_loop;

pub use kpop_flow_a::*;
