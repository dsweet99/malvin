//! Cursor TypeScript SDK backend (`cursor:` models via Node JSONL bridge).
#![cfg_attr(test, allow(unsafe_code))]

mod auth;
pub(crate) mod bridge_path;
pub(crate) mod node_resolve;
mod protocol;
mod session_spawn;

pub use crate::agent_backend::SdkClient as CursorSdkClient;
pub use auth::{effective_sdk_api_key, ensure_sdk_authenticated};
pub(crate) use session_spawn::cursor_spawn_bridge as spawn_bridge;

/// Convenience constructor used by older tests (string model id).
#[must_use]
pub fn cursor_sdk_client_from_raw(
    model: &str,
    io: crate::acp::AgentIoOptions,
    max_retries: u32,
) -> CursorSdkClient {
    let model = crate::model_id::parse_model_id(model).unwrap_or_else(|_| {
        crate::model_id::ParsedModel {
            backend: crate::model_id::ModelBackend::Cursor,
            slug: "auto".into(),
        }
    });
    CursorSdkClient::with_max_retries(
        model,
        crate::agent_backend::BridgeKind::Cursor,
        io,
        max_retries,
    )
}

#[cfg(test)]
pub(crate) async fn session_io_write_cancel_for_test(
    session: &crate::bridge_sdk::BridgeSession,
) -> Result<(), crate::acp::AgentError> {
    crate::bridge_sdk::write_request(
        session,
        &crate::bridge_protocol::BridgeRequest::Cancel {},
    )
    .await
}

#[cfg(test)]
mod kiss_coverage;
#[cfg(test)]
mod session_spawn_tests;
#[cfg(test)]
mod bridge_path_tests;
#[cfg(test)]
mod protocol_tests;
#[cfg(test)]
mod session_mock_tests;
#[cfg(test)]
mod client_mock_tests;
#[cfg(test)]
mod client_ensure_tests;
#[cfg(test)]
mod sdk_bug_helpers;
#[cfg(test)]
mod sdk_bug_regression_tests;
#[cfg(test)]
mod sdk_drain_idle_tests;
#[cfg(test)]
mod sdk_bug2_poison_tests;
