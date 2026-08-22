use std::time::Duration;

use pi::auth::AuthStorage;
use pi::sdk::{Config, ModelRegistry};

use crate::command_output_timeout::timeout_ms_from_env;

pub const DEFAULT_PI_LIST_MODELS_TIMEOUT_MS: u64 = 30_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PiModelListing {
    pub id: String,
    pub name: String,
    pub thinking: Option<bool>,
}

#[must_use]
pub fn pi_list_models_timeout() -> Duration {
    timeout_ms_from_env(
        "MALVIN_PI_LIST_MODELS_TIMEOUT_MS",
        DEFAULT_PI_LIST_MODELS_TIMEOUT_MS,
    )
}

pub fn list_pi_models_sync() -> Result<Vec<PiModelListing>, String> {
    let auth = AuthStorage::load(Config::auth_path()).map_err(|e| e.to_string())?;
    let registry = ModelRegistry::load_for_listing(&auth, None);
    let models: Vec<PiModelListing> = registry
        .models()
        .iter()
        .map(|entry| PiModelListing {
            id: format!("{}/{}", entry.model.provider, entry.model.id),
            name: entry.model.name.clone(),
            thinking: Some(entry.model.reasoning),
        })
        .collect();
    if models.is_empty() {
        return Err("pi model registry produced no models".to_string());
    }
    Ok(models)
}

#[cfg(test)]
#[path = "models_list_tests.rs"]
mod models_list_tests;
