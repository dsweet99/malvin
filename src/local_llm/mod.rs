#![cfg_attr(test, allow(unsafe_code))]

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
mod local_llms_config;
mod local_models_list;
pub(crate) mod provider_metadata;
mod session_spawn_local;

pub(crate) use local_lifecycle::{
    hold_local_llm, local_output_cap, prepare_local_llm, release_local_llm,
};
pub use local_lifecycle::{housekeep_local_llms, model_needs_local_llm};
pub use local_llm_daemon::run_local_llm_manager;
pub use local_llm_paths::INTERNAL_MANAGER_FLAG;
pub use local_models_list::filter_local_listings;
pub use provider_metadata::provider_is_keyless_local;
pub(crate) use session_spawn_local::local_cli_args;

#[must_use]
pub fn local_llm_manager_pid() -> Option<u32> {
    local_llm_lock::lock_holder_pid(&local_llm_paths::manager_lock_path())
}

#[cfg(test)]
mod kiss_coverage_tests;
