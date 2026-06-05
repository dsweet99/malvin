#[path = "gate_kpop_workflow/prepared.rs"]
mod prepared;
#[path = "gate_kpop_workflow/behavior.rs"]
mod behavior;
#[path = "gate_kpop_workflow/params.rs"]
mod params;
#[path = "gate_kpop_workflow/kpop_session.rs"]
mod kpop_session;
#[path = "gate_kpop_workflow/run_loop.rs"]
mod run_loop;

pub(crate) use kpop_session::{fail_gate_kpop_after_exhausted, finish_gate_kpop_after_pass};
pub(crate) use prepared::GateKpopPrepared;
pub(crate) use behavior::GateLoopBehavior;
pub(crate) use params::GateKpopLoopParams;
pub(crate) use run_loop::run_gate_kpop_loop;

#[cfg(test)]
pub(crate) use kpop_session::post_gate_kpop_gates;
#[cfg(test)]
#[path = "gate_kpop_workflow/run_loop_tests.rs"]
pub(crate) mod run_loop_tests;
