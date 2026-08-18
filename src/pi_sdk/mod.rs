
#![cfg_attr(test, allow(unsafe_code))]

mod auth;
mod discover;
mod map_event;
mod map_event_summary;
mod models_list;
mod protocol;
mod session_io;
mod session_spawn;

pub use auth::{ensure_pi_authenticated, is_provider_authenticated};
pub use discover::{pi_missing_binary_message, resolve_pi_bin};
pub use models_list::{
    list_pi_models_sync, pi_list_models_timeout, PiModelListing, DEFAULT_PI_LIST_MODELS_TIMEOUT_MS,
};
pub(crate) use session_io::{pi_send_prompt as send_prompt, pi_write_abort as write_abort};
pub(crate) use session_spawn::pi_spawn_bridge as spawn_bridge;

#[must_use]
pub fn pi_sdk_client_from_raw(
    model: &str,
    io: crate::acp::AgentIoOptions,
    max_retries: u32,
) -> crate::agent_backend::SdkClient {
    let model = crate::model_id::parse_model_id(model).unwrap_or_else(|_| {
        crate::model_id::ParsedModel {
            backend: crate::model_id::ModelBackend::Pi,
            slug: "openai/gpt-4o".into(),
            params: Vec::new(),
        }
    });
    crate::agent_backend::SdkClient::with_max_retries(
        model,
        crate::agent_backend::BridgeKind::Pi,
        io,
        max_retries,
    )
}

#[cfg(test)]
mod client_mock_tests;
#[cfg(test)]
mod discover_tests;
#[cfg(test)]
mod kiss_coverage_tests;
#[cfg(test)]
mod map_event_tests;
#[cfg(test)]
mod protocol_tests;
#[cfg(test)]
mod session_spawn_tests;
