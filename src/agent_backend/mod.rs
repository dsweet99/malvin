mod backend;
mod backend_ops;
mod factory;
mod sdk_client;
mod sdk_client_prompt;
mod sdk_client_session;
mod sdk_session;

#[cfg(test)]
pub(crate) mod test_support;

#[cfg(test)]
#[path = "backend_tests.rs"]
mod backend_tests;

#[cfg(test)]
#[path = "agent_backend_kiss_cov.rs"]
mod agent_backend_kiss_cov;

pub use backend::{AgentBackend, agent_backend_from_client};
pub use backend_ops::{
    agent_backend_attach_run_timing_for_session, agent_backend_ensure_coder_session,
    agent_backend_ensure_run_timing_for_session, agent_backend_set_implement_display_name,
    agent_backend_set_run_timing, agent_backend_timing,
};
pub use factory::{build_agent_backend, build_agent_backend_with_tee};
pub use sdk_client::{BridgeKind, SdkClient};
pub(crate) use sdk_session::SdkSession;
