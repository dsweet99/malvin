//! Static catalog of supported `local:` model ids.

use malvin_mini::ModelListing;

use super::cache::is_model_cached;
use crate::model_id::LOCAL_PREFIX;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalModelSpec {
    /// Slug after `local:` (e.g. `qwen35_9b_q4`).
    pub slug: &'static str,
    /// Human-readable name for `malvin models`.
    pub display_name: &'static str,
    /// Hugging Face repo id used for download.
    pub hf_repo: &'static str,
    /// Directory name under `~/.malvin_home/model_cache/`.
    pub cache_dirname: &'static str,
    /// Loader kind understood by the Python sidecar.
    pub loader: &'static str,
}

pub const LOCAL_MODELS: &[LocalModelSpec] = &[
    LocalModelSpec {
        slug: "qwen35_9b_q4",
        display_name: "Qwen3.5-9B MLX 4-bit",
        hf_repo: "mlx-community/Qwen3.5-9B-MLX-4bit",
        cache_dirname: "Qwen3.5-9B-MLX-4bit",
        loader: "mlx_lm",
    },
    LocalModelSpec {
        slug: "nemotron_cascade2",
        display_name: "Nemotron Cascade 2 JANG_2L",
        hf_repo: "JANGQ-AI/Nemotron-Cascade-2-30B-A3B-JANG_2L",
        cache_dirname: "Nemotron-Cascade-2-30B-A3B-JANG_2L",
        loader: "jang",
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

#[must_use]
pub fn local_model_listings() -> Vec<ModelListing> {
    LOCAL_MODELS
        .iter()
        .map(|spec| {
            let status = if is_model_cached(spec) {
                "cached"
            } else {
                "needs download"
            };
            ModelListing {
                id: spec.slug.to_string(),
                name: format!("{} ({status})", spec.display_name),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_contains_v1_models() {
        assert!(lookup_local_model("qwen35_9b_q4").is_some());
        assert!(lookup_local_model("nemotron_cascade2").is_some());
        assert!(lookup_local_model("missing").is_none());
        assert!(require_known_local_slug("nope").is_err());
        let spec: &LocalModelSpec = lookup_local_model("qwen35_9b_q4").expect("qwen");
        assert_eq!(spec.slug, "qwen35_9b_q4");
        assert!(!spec.display_name.is_empty());
        assert!(spec.hf_repo.contains('/'));
        assert!(!spec.cache_dirname.is_empty());
        assert_eq!(spec.loader, "mlx_lm");
    }

    #[test]
    fn listings_use_local_slugs() {
        let rows = local_model_listings();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].id, "qwen35_9b_q4");
        assert!(rows[0].name.contains("Qwen"));
        let spec = LocalModelSpec {
            slug: "x",
            display_name: "X",
            hf_repo: "org/x",
            cache_dirname: "x",
            loader: "mlx_lm",
        };
        assert_eq!(spec.slug, "x");
        let LocalModelSpec {
            slug,
            display_name,
            hf_repo,
            cache_dirname,
            loader,
        } = *lookup_local_model("qwen35_9b_q4").expect("spec");
        assert_eq!(slug, "qwen35_9b_q4");
        assert!(!display_name.is_empty());
        assert!(hf_repo.contains('/'));
        assert!(!cache_dirname.is_empty());
        assert_eq!(loader, "mlx_lm");
    }
}
