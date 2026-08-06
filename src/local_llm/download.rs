//! Download local GGUF models into `~/.malvin_home/model_cache`.

use std::path::{Path, PathBuf};
use std::process::Command;

use super::cache::{is_model_cached, model_cache_dir, model_cache_path, model_cache_root};
use super::registry::{require_known_local_slug, LocalModelSpec};
use crate::model_id::{parse_model_id, MiniTransport, ModelBackend, MINI_PREFIX};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadPolicy {
    Allow,
    Deny,
}

pub fn resolve_download_target(raw: &str) -> Result<&'static LocalModelSpec, String> {
    let raw = raw.trim();
    let slug = match parse_model_id(raw) {
        Ok(parsed) if parsed.backend == ModelBackend::Mini(MiniTransport::Local) => parsed.slug,
        Ok(_) => {
            return Err(format!(
                "download only supports `{MINI_PREFIX}local/<id>` models (got `{raw}`)"
            ));
        }
        Err(e) => {
            if !raw.contains(':') {
                return Err(format!(
                    "download only supports `{MINI_PREFIX}local/<id>` (got bare `{raw}`); try `{MINI_PREFIX}local/{raw}`"
                ));
            }
            return Err(e);
        }
    };
    require_known_local_slug(&slug)
}

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

pub fn ensure_model_cached(
    spec: &LocalModelSpec,
    policy: DownloadPolicy,
) -> Result<PathBuf, String> {
    if is_model_cached(spec) {
        return Ok(model_cache_path(spec));
    }
    if policy == DownloadPolicy::Deny {
        return Err(format!(
            "local model `{MINI_PREFIX}local/{}` is not cached at {} (pass without --no-download, or run `malvin models download {MINI_PREFIX}local/{}`)",
            spec.slug,
            model_cache_path(spec).display(),
            spec.slug
        ));
    }
    download_local_model(&format!("{MINI_PREFIX}local/{}", spec.slug))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_download_target_accepts_mini_local_only() {
        assert_eq!(
            resolve_download_target("mini:local/qwen35_9b_q4")
                .expect("ok")
                .slug,
            "qwen35_9b_q4"
        );
        assert!(resolve_download_target("nemotron3_nano_4b")
            .expect_err("bare")
            .contains("mini:local/"));
        assert!(resolve_download_target("local:qwen35_9b_q4").is_err());
        assert!(resolve_download_target("mini:openrouter/x").is_err());
    }
}
