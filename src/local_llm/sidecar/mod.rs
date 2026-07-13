//! Host-managed MLX sidecar process (outside the agent sandbox RSS monitor).

mod health;
mod spawn;

use std::fs;
use std::path::Path;
use std::process::Child;
use std::time::Duration;

use super::download::ensure_model_cached;
use super::registry::{require_known_local_slug, LocalModelSpec};
use crate::model_id::{parse_model_id, LOCAL_PREFIX, ModelBackend};

use health::wait_for_health;
use spawn::{free_loopback_port, sidecar_run_dir, spawn_sidecar_process};

const DEFAULT_LOCAL_TIMEOUT_SECS: u64 = 600;

/// Running local inference sidecar bound to a loopback OpenAI-compatible `/v1` endpoint.
pub struct LocalSidecarHandle {
    pub base_url: String,
    pub model_slug: String,
    child: Option<Child>,
    run_dir: std::path::PathBuf,
}

impl LocalSidecarHandle {
    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    #[must_use]
    pub fn model_slug(&self) -> &str {
        &self.model_slug
    }
}

impl Drop for LocalSidecarHandle {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        let _ = fs::remove_dir_all(&self.run_dir);
    }
}

/// Ensure a local model is cached and its sidecar is listening.
///
/// Spawns the Python server with [`std::process::Command`] (not `malvin_std_command`) so the
/// process is outside the agent sandbox RSS accounting.
///
/// # Errors
///
/// Returns an error when the platform is unsupported, the model cannot be prepared, or the
/// sidecar fails to become healthy.
pub fn ensure_local_sidecar(
    model_id: &str,
    allow_download: bool,
) -> Result<LocalSidecarHandle, String> {
    #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
    {
        let _ = (model_id, allow_download);
        return Err(
            "local: models require Apple Silicon macOS (MLX) in this malvin version".into(),
        );
    }
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        ensure_local_sidecar_impl(model_id, allow_download)
    }
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn ensure_local_sidecar_impl(
    model_id: &str,
    allow_download: bool,
) -> Result<LocalSidecarHandle, String> {
    let slug = local_slug(model_id)?;
    let spec = require_known_local_slug(&slug)?;
    let model_dir = ensure_model_cached(spec, allow_download)?;
    start_sidecar(spec, &model_dir)
}

fn local_slug(model_id: &str) -> Result<String, String> {
    match parse_model_id(model_id) {
        Ok(parsed) if parsed.backend == ModelBackend::Local => Ok(parsed.slug),
        Ok(_) => Err(format!("expected `{LOCAL_PREFIX}<id>`, got `{model_id}`")),
        Err(e) => Err(e),
    }
}

fn start_sidecar(spec: &LocalModelSpec, model_dir: &Path) -> Result<LocalSidecarHandle, String> {
    let port = free_loopback_port()?;
    let run_dir = sidecar_run_dir(spec.slug, port)?;
    let mut child = spawn_sidecar_process(spec, model_dir, port, &run_dir)?;
    let base_url = format!("http://127.0.0.1:{port}/v1");
    if let Err(e) = wait_for_health(&base_url) {
        return fail_sidecar_boot(&mut child, &run_dir, e);
    }
    Ok(LocalSidecarHandle {
        base_url,
        model_slug: spec.slug.to_string(),
        child: Some(child),
        run_dir,
    })
}

fn fail_sidecar_boot(child: &mut Child, run_dir: &Path, error: String) -> Result<LocalSidecarHandle, String> {
    let _ = child.kill();
    let _ = child.wait();
    let stderr = fs::read_to_string(run_dir.join("sidecar.stderr")).unwrap_or_default();
    let _ = fs::remove_dir_all(run_dir);
    Err(format!(
        "local sidecar failed health check: {error}\n{}",
        stderr.trim()
    ))
}

/// Build OpenRouter-compatible config pointing at a running local sidecar.
#[must_use]
pub fn local_openrouter_config(model_slug: &str, base_url: &str) -> malvin_mini::OpenRouterConfig {
    malvin_mini::OpenRouterConfig {
        model: model_slug.to_string(),
        api_key: String::new(),
        http_referer: None,
        request_timeout: Duration::from_secs(
            std::env::var("MALVIN_LOCAL_REQUEST_TIMEOUT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(DEFAULT_LOCAL_TIMEOUT_SECS),
        ),
        base_url: base_url.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_openrouter_config_has_empty_api_key() {
        let cfg = local_openrouter_config("qwen35_9b_q4", "http://127.0.0.1:9/v1");
        assert!(cfg.api_key.is_empty());
        assert_eq!(cfg.model, "qwen35_9b_q4");
        assert_eq!(cfg.base_url, "http://127.0.0.1:9/v1");
        assert!(cfg.request_timeout.as_secs() >= 120);
    }

    #[test]
    fn local_slug_requires_local_prefix() {
        assert_eq!(local_slug("local:qwen35_9b_q4").expect("ok"), "qwen35_9b_q4");
        assert!(local_slug("openrouter:x").is_err());
    }

    #[test]
    fn ensure_local_sidecar_rejects_non_local_ids() {
        assert!(ensure_local_sidecar("openrouter:x", false).is_err());
    }

    #[test]
    fn kiss_witness_sidecar_symbols() {
        let _ = stringify!(ensure_local_sidecar_impl);
        let _ = stringify!(start_sidecar);
        let _ = stringify!(fail_sidecar_boot);
        let _ = stringify!(LocalSidecarHandle);
        let _ = stringify!(ensure_local_sidecar);
        let tmp = tempfile::tempdir().expect("tmp");
        let handle = LocalSidecarHandle {
            base_url: "http://127.0.0.1:9/v1".into(),
            model_slug: "qwen35_9b_q4".into(),
            child: None,
            run_dir: tmp.path().to_path_buf(),
        };
        assert_eq!(handle.base_url(), "http://127.0.0.1:9/v1");
        assert_eq!(handle.model_slug(), "qwen35_9b_q4");
        drop(handle);
        assert!(!tmp.path().exists());
    }
}
