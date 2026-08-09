//! Local GGUF model registry, cache, download, and llama.cpp engine for
//! `mini:local/…` and `prime:local/local/…`.

mod cache;
mod download;
mod engine;
mod openai_http;
mod openai_server;
mod prime_local_sidecar;
mod prime_models_json;
mod registry;

pub use cache::{is_model_cached, model_cache_dir, model_cache_path, model_cache_root};
pub use download::{download_local_model, ensure_model_cached, DownloadPolicy};
pub use engine::{ensure_local_engine, LocalCompletionEngine};
pub use openai_server::LocalOpenAiServer;
pub use prime_local_sidecar::PrimeLocalSidecar;
pub use registry::{
    local_backend_supported, lookup_local_model, local_model_listings, require_known_local_slug,
    LocalModelSpec,
};

#[cfg(test)]
#[path = "registry_kiss_cov_tests.rs"]
mod registry_kiss_cov_tests;

