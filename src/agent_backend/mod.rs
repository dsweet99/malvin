pub mod backend_error_tracker;
mod backend_lifecycle;
mod factory;
mod sdk_client;
mod sdk_client_active;
mod sdk_client_header_lifecycle;
mod sdk_client_prompt;
mod sdk_client_session;
mod sdk_client_session_header;
#[cfg(test)]
pub(crate) use sdk_client_session_header::header_prompt_options_for_test;
pub use sdk_client_session_header::{pending_session_header, session_header_is_satisfied};
mod sdk_session;

#[cfg(test)]
pub(crate) mod test_support;

#[cfg(test)]
#[path = "backend_tests.rs"]
mod backend_tests;

#[cfg(test)]
#[path = "backend_error_tracker_tests.rs"]
mod backend_error_tracker_tests;

#[cfg(test)]
#[path = "agent_backend_kiss_cov.rs"]
mod agent_backend_kiss_cov;

pub use backend_error_tracker::{BackendErrorTracker, MAX_CONSECUTIVE_SAME_BACKEND_ERRORS};
pub use factory::{
    AgentStdoutTeeFlags, agent_io_options, build_agent_backend, build_agent_backend_with_tee,
    default_workflow_stdout_tee_flags,
};
pub use sdk_client::{SdkClient, ensure_run_timing_for_session, set_implement_display_name};
#[cfg(test)]
pub(crate) use sdk_client::{begun_cwd, live_session, live_session_mut, new_cursor, new_pi};
pub use sdk_client_active::ActiveCoderSession;
pub use sdk_client_session::CoderSessionEnsure;
#[cfg(test)]
pub(crate) use sdk_client_session::sdk_bridge_needs_restart;
pub(crate) use sdk_session::SdkSession;
