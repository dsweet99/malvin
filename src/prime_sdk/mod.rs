//! Prime SDK backend (`prime:` models via Node JSONL bridge to `prime-agent`).
#![cfg_attr(test, allow(unsafe_code))]

mod auth;
pub(crate) mod bridge_path;
mod models_list;
pub(crate) mod node_resolve;
mod protocol;
mod session_spawn;

pub use crate::agent_backend::SdkClient as PrimeSdkClient;
pub use auth::{effective_prime_api_key, ensure_prime_authenticated};
pub use models_list::{list_prime_models_sync, PrimeModelListing};
pub(crate) use session_spawn::prime_spawn_bridge as spawn_bridge;

/// Convenience constructor used by older tests (string model id).
#[must_use]
pub fn prime_sdk_client_from_raw(
    model: &str,
    io: crate::acp::AgentIoOptions,
    max_retries: u32,
) -> PrimeSdkClient {
    let model = crate::model_id::parse_model_id(model).unwrap_or_else(|_| {
        crate::model_id::ParsedModel {
            backend: crate::model_id::ModelBackend::Prime,
            slug: "openai/gpt-4o".into(),
        }
    });
    let mut client = PrimeSdkClient::with_max_retries(
        model,
        crate::agent_backend::BridgeKind::Prime,
        io,
        max_retries,
    );
    client.allow_download = true;
    client
}

#[cfg(test)]
mod client_mock_tests;
#[cfg(test)]
mod kiss_coverage_tests;
#[cfg(test)]
mod protocol_tests;
#[cfg(test)]
mod timing_tests;
#[cfg(test)]
mod session_io_tests;
#[cfg(test)]
mod session_spawn_tests;
#[cfg(test)]
mod log_adapter_tests;
#[cfg(test)]
mod node_resolve_tests;
