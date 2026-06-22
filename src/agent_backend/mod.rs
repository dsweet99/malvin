mod backend;
mod backend_ops;
mod factory;
mod kpop_bridge;
pub mod mini;
#[cfg(test)]
mod test_support;

pub use backend::AgentBackend;
pub use backend_ops::{
    agent_backend_attach_run_timing_for_session, agent_backend_ensure_run_timing_for_session,
    agent_backend_run_kpop_flow, agent_backend_run_kpop_multiturn,
    agent_backend_set_implement_display_name, agent_backend_set_run_timing, agent_backend_timing,
};
pub use factory::{build_agent_backend, build_agent_backend_with_tee};
#[cfg(test)]
#[path = "backend_kpop_test.rs"]
mod backend_kpop_test;
