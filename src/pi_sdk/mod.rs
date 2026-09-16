#![cfg_attr(test, allow(unsafe_code))]

mod auth;
mod cache_clock;
mod isolated_bash;
mod local_context;
mod local_endpoint;
mod local_lifecycle;
mod local_llm_client;
mod local_llm_daemon;
mod local_llm_keepalive;
mod local_llm_lock;
mod local_llm_ollama;
mod local_llm_paths;
mod local_llm_protocol;
#[cfg(test)]
mod local_llm_test_lock;
mod map_agent_event;
mod map_agent_event_end;
mod map_event_summary;
mod models_list;
mod models_refresh;
mod models_refresh_merge;
mod openrouter_billed_cost;
mod openrouter_pricing;
mod runtime;
mod session;
mod session_fake;
mod session_spawn;
mod session_spawn_watch;
mod usage_cost;

pub use auth::{ensure_pi_authenticated, is_provider_authenticated, is_provider_listable};
pub use local_lifecycle::{housekeep_local_llms, model_needs_local_llm};
pub use local_llm_daemon::run_local_llm_manager;
pub use local_llm_paths::INTERNAL_MANAGER_FLAG;

#[must_use]
pub fn local_llm_manager_pid() -> Option<u32> {
    local_llm_lock::lock_holder_pid(&local_llm_paths::manager_lock_path())
}
pub use models_list::{
    DEFAULT_PI_LIST_MODELS_TIMEOUT_MS, PiModelListing, list_pi_models_sync, pi_list_models_timeout,
};
pub use models_refresh::refresh_pi_provider_caches_if_stale;

#[allow(dead_code)]
const _: fn() = || {
    let _ = std::any::type_name::<pi::sdk::SessionOptions>();
};
pub(crate) use session::PiEmbeddedSession;
pub(crate) use session_spawn::pi_spawn_bridge as spawn_bridge;
#[cfg(test)]
pub(crate) use session_spawn::{local_append_system_prompt, local_enabled_tools};

#[must_use]
pub fn pi_sdk_client_from_raw(
    model: &str,
    io: crate::acp::AgentIoOptions,
    max_retries: u32,
) -> crate::agent_backend::SdkClient {
    let model = crate::model_id::parse_model_id(model)
        .unwrap_or_else(|e| panic!("pi_sdk_client_from_raw requires a valid model id: {e}"));
    crate::agent_backend::SdkClient::with_max_retries(model, io, max_retries)
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
