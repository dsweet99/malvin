//! Static catalog of supported `local:` GGUF model ids.

use crate::malvin_mini::ModelListing;

use crate::model_id::LOCAL_PREFIX;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalModelSpec {
    /// Slug after `local:` (e.g. `qwen35_9b_q4`).
    pub slug: &'static str,
    /// Human-readable name for `malvin models`.
    pub display_name: &'static str,
    /// Hugging Face repo id.
    pub hf_repo: &'static str,
    /// Directory name under `~/.malvin_home/model_cache/`.
    pub cache_dirname: &'static str,
    /// GGUF filename within the cache directory.
    pub gguf_filename: &'static str,
    /// Direct resolve URL for the GGUF artifact.
    pub resolve_url: &'static str,
    /// Minimum `mem_limit_gb` for in-sandbox load (weights + runtime headroom).
    pub min_mem_limit_gb: u64,
}

pub const LOCAL_MODELS: &[LocalModelSpec] = &[
    LocalModelSpec {
        slug: "qwen35_9b_q4",
        display_name: "Qwen3.5-9B Q4_K_M GGUF",
        hf_repo: "unsloth/Qwen3.5-9B-GGUF",
        cache_dirname: "Qwen3.5-9B-GGUF",
        gguf_filename: "Qwen3.5-9B-Q4_K_M.gguf",
        resolve_url: "https://huggingface.co/unsloth/Qwen3.5-9B-GGUF/resolve/main/Qwen3.5-9B-Q4_K_M.gguf",
        // ~5.3 GiB on disk; Metal residency needs headroom above the GGUF size.
        min_mem_limit_gb: 8,
    },
    LocalModelSpec {
        slug: "nemotron3_nano_4b",
        display_name: "Nemotron-3-Nano-4B Q4_K_M GGUF",
        hf_repo: "nvidia/NVIDIA-Nemotron-3-Nano-4B-GGUF",
        cache_dirname: "NVIDIA-Nemotron-3-Nano-4B-GGUF",
        gguf_filename: "NVIDIA-Nemotron3-Nano-4B-Q4_K_M.gguf",
        resolve_url:
            "https://huggingface.co/nvidia/NVIDIA-Nemotron-3-Nano-4B-GGUF/resolve/main/NVIDIA-Nemotron3-Nano-4B-Q4_K_M.gguf",
        // ~2.6 GiB on disk; keep margin for context + agent USS.
        min_mem_limit_gb: 6,
    },
];

#[must_use]
pub fn lookup_local_model(slug: &str) -> Option<&'static LocalModelSpec> {
    LOCAL_MODELS.iter().find(|m| m.slug == slug)
}

/// # Errors
///
/// Returns an error when `slug` is not in the local catalog.
pub fn require_known_local_slug(slug: &str) -> Result<&'static LocalModelSpec, String> {
    lookup_local_model(slug).ok_or_else(|| {
        let known = LOCAL_MODELS
            .iter()
            .map(|m| format!("{LOCAL_PREFIX}{}", m.slug))
            .collect::<Vec<_>>()
            .join(", ");
        format!("unknown local model `{LOCAL_PREFIX}{slug}`; known: {known}")
    })
}

/// True when `local:` models can run (Apple Silicon Metal). Used to omit them from listings.
#[must_use]
pub const fn local_backend_supported() -> bool {
    crate::malvin_llama::metal_backend_supported()
}

/// Listings for `malvin models`. Empty when Metal is unavailable (no runnable local backend).
#[must_use]
pub fn local_model_listings() -> Vec<ModelListing> {
    if !local_backend_supported() {
        return Vec::new();
    }
    LOCAL_MODELS
        .iter()
        .map(|spec| ModelListing {
            id: spec.slug.to_string(),
            name: spec.display_name.to_string(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_contains_v1_gguf_models() {
        assert!(lookup_local_model("qwen35_9b_q4").is_some());
        assert!(lookup_local_model("nemotron3_nano_4b").is_some());
        assert!(lookup_local_model("nemotron_cascade2").is_none());
        assert!(lookup_local_model("missing").is_none());
        assert!(require_known_local_slug("nope").is_err());
        let spec = lookup_local_model("qwen35_9b_q4").expect("qwen");
        assert!(
            std::path::Path::new(spec.gguf_filename)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("gguf"))
        );
        assert!(spec.resolve_url.starts_with("https://"));
        assert!(spec.hf_repo.contains('/'));
    }

    #[test]
    fn listings_have_no_cache_status() {
        let rows = local_model_listings();
        if local_backend_supported() {
            assert_eq!(rows.len(), 2);
            assert_eq!(rows[0].id, "qwen35_9b_q4");
            assert!(!rows[0].name.contains("cached"));
            assert!(!rows[0].name.contains("download"));
            assert_eq!(rows[1].id, "nemotron3_nano_4b");
        } else {
            assert!(
                rows.is_empty(),
                "non-Metal hosts must omit local: listings"
            );
        }
    }

    #[test]
    fn local_backend_supported_matches_metal_compile_gate() {
        assert_eq!(
            local_backend_supported(),
            cfg!(all(target_os = "macos", target_arch = "aarch64"))
        );
    }

    #[test]
    fn min_mem_floors_exceed_gguf_disk_size_gb() {
        let qwen = lookup_local_model("qwen35_9b_q4").expect("qwen");
        let nano = lookup_local_model("nemotron3_nano_4b").expect("nano");
        assert!(qwen.min_mem_limit_gb >= 8);
        assert!(nano.min_mem_limit_gb >= 6);
        assert!(qwen.min_mem_limit_gb > nano.min_mem_limit_gb);
    }
}
