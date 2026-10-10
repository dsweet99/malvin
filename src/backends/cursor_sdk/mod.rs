#![cfg_attr(test, allow(unsafe_code))]

pub(crate) use super::cursor_sdk_auth as auth;
pub use super::cursor_sdk_bridge_install as bridge_install;
pub(crate) use super::cursor_sdk_bridge_install_npm as bridge_install_npm;
pub use super::cursor_sdk_bridge_path as bridge_path;
pub(crate) use super::cursor_sdk_bridge_stderr as bridge_stderr;
pub use super::cursor_sdk_node_resolve as node_resolve;
pub use super::cursor_sdk_protocol as protocol;
pub(crate) use super::cursor_sdk_session_spawn as session_spawn;

pub use crate::backends::agent_backend::SdkClient as CursorSdkClient;
pub use auth::{effective_sdk_api_key, ensure_sdk_authenticated};
pub(crate) use session_spawn::cursor_spawn_bridge as spawn_bridge;

#[must_use]
pub fn cursor_sdk_client_from_raw(
    model: &str,
    io: crate::agent_process::AgentIoOptions,
    max_retries: u32,
) -> CursorSdkClient {
    let model = crate::config::model_id::parse_model_id(model)
        .unwrap_or_else(|e| panic!("cursor_sdk_client_from_raw requires a valid model id: {e}"));
    CursorSdkClient::with_max_retries(model, io, max_retries)
}

#[cfg(test)]
pub(crate) async fn session_io_write_cancel_for_test(
    session: &crate::backends::bridge_sdk::BridgeSession,
) -> Result<(), crate::agent_process::AgentError> {
    crate::backends::bridge_sdk::write_request(
        session,
        &crate::backends::cursor_sdk::protocol::BridgeRequest::Cancel {},
    )
    .await
}

#[cfg(test)]
mod bridge_install_tests;
#[cfg(test)]
mod bridge_path_tests;
#[cfg(test)]
mod client_ensure_tests;
#[cfg(test)]
mod client_mock_retry_tests;
#[cfg(test)]
mod client_mock_tests;
#[cfg(test)]
mod kiss_coverage;
#[cfg(test)]
mod protocol_tests;
#[cfg(test)]
mod sdk_bug2_poison_tests;
#[cfg(test)]
mod sdk_bug_helpers;
#[cfg(test)]
mod sdk_bug_regression_tests;
#[cfg(test)]
mod sdk_drain_idle_tests;
#[cfg(test)]
mod sdk_drain_progress_tests;
#[cfg(test)]
mod session_mock_tests;
#[cfg(test)]
mod session_spawn_tests;
