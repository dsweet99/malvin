//! Local MLX model registry, cache, download, and sidecar lifecycle for `local:` models.

mod cache;
mod download;
mod registry;
mod sidecar;

pub use cache::{is_model_cached, model_cache_dir, model_cache_root};
pub use download::download_local_model;
pub use registry::{
    lookup_local_model, local_model_listings, require_known_local_slug, LocalModelSpec,
};
pub use sidecar::{ensure_local_sidecar, local_openrouter_config, LocalSidecarHandle};

#[cfg(test)]
#[path = "registry_kiss_cov_tests.rs"]
mod registry_kiss_cov_tests;

#[cfg(test)]
#[path = "sidecar_kiss_cov_tests.rs"]
mod sidecar_kiss_cov_tests;
