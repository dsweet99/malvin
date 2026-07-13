//! `~/.malvin_home/model_cache` layout for local models.

use std::path::PathBuf;

use super::registry::LocalModelSpec;
use crate::workspace_paths::malvin_user_home_root;

pub const MODEL_CACHE_DIR: &str = "model_cache";

#[must_use]
pub fn model_cache_root() -> PathBuf {
    malvin_user_home_root().join(MODEL_CACHE_DIR)
}

#[must_use]
pub fn model_cache_dir(spec: &LocalModelSpec) -> PathBuf {
    model_cache_root().join(spec.cache_dirname)
}

/// A model is cached when its directory exists and contains `config.json`.
#[must_use]
pub fn is_model_cached(spec: &LocalModelSpec) -> bool {
    let dir = model_cache_dir(spec);
    dir.is_dir() && dir.join("config.json").is_file()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::local_llm::registry::lookup_local_model;

    #[test]
    fn cache_paths_under_malvin_home() {
        let spec = lookup_local_model("qwen35_9b_q4").expect("spec");
        let root = model_cache_root();
        assert!(root.ends_with(MODEL_CACHE_DIR));
        assert_eq!(model_cache_dir(spec), root.join(spec.cache_dirname));
    }
}
