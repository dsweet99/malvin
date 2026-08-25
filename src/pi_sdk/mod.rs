#![cfg_attr(test, allow(unsafe_code))]

mod cache_clock;
mod auth;
mod isolated_bash;
mod map_agent_event;
mod map_agent_event_end;
mod map_event_summary;
mod models_list;
mod models_refresh;
mod runtime;
mod session;
mod session_fake;
mod session_spawn;
mod openrouter_pricing;
mod openrouter_billed_cost;
mod usage_cost;

pub use auth::{ensure_pi_authenticated, is_provider_authenticated};
pub use models_list::{
    DEFAULT_PI_LIST_MODELS_TIMEOUT_MS, PiModelListing, list_pi_models_sync, pi_list_models_timeout,
};

/// Compile-only probe: fail CI if `pi_agent_rust` does not export `SessionOptions`.
#[allow(dead_code)]
const _: fn() = || {
    let _ = std::any::type_name::<pi::sdk::SessionOptions>();
};
pub(crate) use session::PiEmbeddedSession;
pub(crate) use session_spawn::pi_spawn_bridge as spawn_bridge;

#[must_use]
pub fn pi_sdk_client_from_raw(
    model: &str,
    io: crate::acp::AgentIoOptions,
    max_retries: u32,
) -> crate::agent_backend::SdkClient {
    let model =
        crate::model_id::parse_model_id(model).unwrap_or_else(|_| crate::model_id::ParsedModel {
            backend: crate::model_id::ModelBackend::Pi,
            slug: "openai/gpt-4o".into(),
            params: Vec::new(),
        });
    crate::agent_backend::SdkClient::with_max_retries(
        model,
        io,
        max_retries,
    )
}

#[cfg(test)]
mod client_mock_tests;
#[cfg(test)]
mod kiss_coverage_tests;
#[cfg(test)]
mod map_agent_event_tests;
#[cfg(test)]
mod session_drain_race_tests;
#[cfg(test)]
mod session_spawn_tests;
