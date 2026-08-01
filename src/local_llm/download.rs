//! Download local GGUF models into `~/.malvin_home/model_cache`.

use std::path::{Path, PathBuf};
use std::process::Command;

use super::cache::{is_model_cached, model_cache_dir, model_cache_path, model_cache_root};
use super::registry::{require_known_local_slug, LocalModelSpec};
use crate::model_id::{parse_model_id, LOCAL_PREFIX, ModelBackend};

/// Whether missing models may be fetched automatically.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadPolicy {
    Allow,
    Deny,
}

/// Resolve `local:<slug>` or a bare slug to a catalog entry.
///
/// # Errors
///
/// Returns an error when the id is not a known local model.
pub fn resolve_download_target(raw: &str) -> Result<&'static LocalModelSpec, String> {
    let raw = raw.trim();
    let slug = match parse_model_id(raw) {
        Ok(parsed) if parsed.backend == ModelBackend::Local => parsed.slug,
        Ok(_) => {
            return Err(format!(
                "download only supports `{LOCAL_PREFIX}<id>` models (got `{raw}`)"
            ));
        }
        Err(_) if !raw.contains(':') => raw.to_string(),
        Err(e) => return Err(e),
    };
    require_known_local_slug(&slug)
}

/// Download a local model into the cache (no-op when already cached).
///
/// # Errors
///
/// Returns an error when curl fails or the model remains uncached.
pub fn download_local_model(raw: &str) -> Result<PathBuf, String> {
    let spec = resolve_download_target(raw)?;
    if is_model_cached(spec) {
        return Ok(model_cache_path(spec));
    }
    ensure_cache_dir(spec)?;
    curl_download(spec.resolve_url, &model_cache_path(spec))?;
    require_cached(spec)
}

fn ensure_cache_dir(spec: &LocalModelSpec) -> Result<(), String> {
    std::fs::create_dir_all(model_cache_root()).map_err(|e| {
        format!(
            "failed to create model cache {}: {e}",
            model_cache_root().display()
        )
    })?;
    std::fs::create_dir_all(model_cache_dir(spec)).map_err(|e| {
        format!(
            "failed to create model dir {}: {e}",
            model_cache_dir(spec).display()
        )
    })
}

fn curl_download(url: &str, dest: &Path) -> Result<(), String> {
    let tmp = dest.with_extension("gguf.partial");
    let status = Command::new("curl")
        .args(["-fL", "--retry", "3", "--retry-delay", "2", "-o"])
        .arg(&tmp)
        .arg(url)
        .status()
        .map_err(|e| format!("failed to spawn curl: {e}"))?;
    if !status.success() {
        let _ = std::fs::remove_file(&tmp);
        return Err(format!("curl download failed for {url} (exit {status})"));
    }
    std::fs::rename(&tmp, dest).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        format!("failed to finalize {}: {e}", dest.display())
    })
}

fn require_cached(spec: &LocalModelSpec) -> Result<PathBuf, String> {
    if is_model_cached(spec) {
        Ok(model_cache_path(spec))
    } else {
        Err(format!(
            "download finished but GGUF missing at {}",
            model_cache_path(spec).display()
        ))
    }
}

/// Ensure the model is cached, downloading unless policy is [`DownloadPolicy::Deny`].
///
/// # Errors
///
/// Returns an error when the model is missing and download is denied or fails.
pub fn ensure_model_cached(
    spec: &LocalModelSpec,
    policy: DownloadPolicy,
) -> Result<PathBuf, String> {
    if is_model_cached(spec) {
        return Ok(model_cache_path(spec));
    }
    if policy == DownloadPolicy::Deny {
        return Err(format!(
            "local model `{LOCAL_PREFIX}{}` is not cached at {} (pass without --no-download, or run `malvin models download {LOCAL_PREFIX}{}`)",
            spec.slug,
            model_cache_path(spec).display(),
            spec.slug
        ));
    }
    download_local_model(&format!("{LOCAL_PREFIX}{}", spec.slug))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_download_target_accepts_prefixed_and_bare() {
        assert_eq!(
            resolve_download_target("local:qwen35_9b_q4")
                .expect("ok")
                .slug,
            "qwen35_9b_q4"
        );
        assert_eq!(
            resolve_download_target("nemotron3_nano_4b")
                .expect("ok")
                .slug,
            "nemotron3_nano_4b"
        );
        assert!(resolve_download_target("openrouter:x").is_err());
        assert!(resolve_download_target("local:unknown").is_err());
        assert!(resolve_download_target("nemotron_cascade2").is_err());
    }

    #[test]
    fn ensure_model_cached_errors_when_download_disabled() {
        let spec = require_known_local_slug("qwen35_9b_q4").expect("spec");
        if is_model_cached(spec) {
            return;
        }
        let err = ensure_model_cached(spec, DownloadPolicy::Deny).expect_err("no download");
        assert!(err.contains("--no-download") || err.contains("not cached"));
    }
}
