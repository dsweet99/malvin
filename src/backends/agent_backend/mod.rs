pub(crate) use super::agent_backend_acp_attempt_loop as acp_attempt_loop;
pub(crate) use super::agent_backend_backend_error_stop as backend_error_stop;
pub use super::agent_backend_backend_error_tracker as backend_error_tracker;
pub(crate) use super::agent_backend_backend_lifecycle as backend_lifecycle;
pub(crate) use super::agent_backend_factory as factory;
pub(crate) use super::agent_backend_sdk_client as sdk_client;
pub(crate) use super::agent_backend_sdk_client_active as sdk_client_active;
pub(crate) use super::agent_backend_sdk_client_header_lifecycle as sdk_client_header_lifecycle;
pub(crate) use super::agent_backend_sdk_client_prompt as sdk_client_prompt;
pub(crate) use super::agent_backend_sdk_client_session as sdk_client_session;
pub(crate) use super::agent_backend_sdk_client_session_header as sdk_client_session_header;
pub(crate) use super::agent_backend_sdk_session as sdk_session;
#[cfg(test)]
pub(crate) use sdk_client_session_header::header_prompt_options_for_test;
pub use sdk_client_session_header::{pending_session_header, session_header_is_satisfied};

#[cfg(test)]
pub(crate) mod test_support;

#[cfg(test)]
#[path = "backend_tests.rs"]
mod backend_tests;

#[cfg(test)]
#[path = "backend_error_tracker_tests.rs"]
mod backend_error_tracker_tests;

#[cfg(test)]
#[path = "backend_error_stop_tests.rs"]
mod backend_error_stop_tests;

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
pub(crate) use sdk_client::{begun_cwd, live_session, live_session_mut};
pub use sdk_client_active::ActiveCoderSession;
pub use sdk_client_session::CoderSessionEnsure;
#[cfg(test)]
pub(crate) use sdk_client_session::sdk_bridge_needs_restart;
