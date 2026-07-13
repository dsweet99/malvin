//! Download local models into `~/.malvin_home/model_cache` via the Python helper.

use std::path::PathBuf;
use std::process::{Command, ExitStatus};

use super::cache::{is_model_cached, model_cache_dir, model_cache_root};
use super::registry::{require_known_local_slug, LocalModelSpec};
use crate::model_id::{parse_model_id, LOCAL_PREFIX, ModelBackend};

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
/// Returns an error when the helper script fails or the model remains uncached.
pub fn download_local_model(raw: &str) -> Result<PathBuf, String> {
    let spec = resolve_download_target(raw)?;
    if is_model_cached(spec) {
        return Ok(model_cache_dir(spec));
    }
    ensure_cache_root()?;
    run_download_script(spec)?;
    require_cached(spec)
}

fn ensure_cache_root() -> Result<(), String> {
    std::fs::create_dir_all(model_cache_root()).map_err(|e| {
        format!(
            "failed to create model cache {}: {e}",
            model_cache_root().display()
        )
    })
}

fn run_download_script(spec: &LocalModelSpec) -> Result<(), String> {
    let script = local_llm_script("download.py")?;
    let python = resolve_python()?;
    let status = Command::new(&python)
        .arg(&script)
        .arg("--repo")
        .arg(spec.hf_repo)
        .arg("--out")
        .arg(model_cache_dir(spec))
        .arg("--loader")
        .arg(spec.loader)
        .status()
        .map_err(|e| format!("failed to spawn {}: {e}", python.display()))?;
    check_download_status(spec, status)
}

fn check_download_status(spec: &LocalModelSpec, status: ExitStatus) -> Result<(), String> {
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "local model download failed for `{LOCAL_PREFIX}{}` (exit {status})",
            spec.slug
        ))
    }
}

fn require_cached(spec: &LocalModelSpec) -> Result<PathBuf, String> {
    if is_model_cached(spec) {
        Ok(model_cache_dir(spec))
    } else {
        Err(format!(
            "download finished but cache is incomplete at {}",
            model_cache_dir(spec).display()
        ))
    }
}

/// Ensure the model is cached, downloading unless `allow_download` is false.
///
/// # Errors
///
/// Returns an error when the model is missing and download is disabled or fails.
pub fn ensure_model_cached(spec: &LocalModelSpec, allow_download: bool) -> Result<PathBuf, String> {
    if is_model_cached(spec) {
        return Ok(model_cache_dir(spec));
    }
    if !allow_download {
        return Err(format!(
            "local model `{LOCAL_PREFIX}{}` is not cached at {} (pass without --no-download, or run `malvin models download {LOCAL_PREFIX}{}`)",
            spec.slug,
            model_cache_dir(spec).display(),
            spec.slug
        ));
    }
    download_local_model(&format!("{LOCAL_PREFIX}{}", spec.slug))
}

pub(crate) fn local_llm_script(name: &str) -> Result<PathBuf, String> {
    if let Some(path) = env_override_script(name) {
        return Ok(path);
    }
    for path in script_candidates(name) {
        if path.is_file() {
            return Ok(path);
        }
    }
    Err(format!(
        "could not find scripts/local_llm/{name}; set MALVIN_LOCAL_LLM_DIR"
    ))
}

fn env_override_script(name: &str) -> Option<PathBuf> {
    let override_dir = std::env::var_os("MALVIN_LOCAL_LLM_DIR")?;
    let candidate = PathBuf::from(override_dir).join(name);
    candidate.is_file().then_some(candidate)
}

fn script_candidates(name: &str) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join("local_llm").join(name));
            candidates.push(dir.join("../scripts/local_llm").join(name));
            candidates.push(dir.join("../../scripts/local_llm").join(name));
        }
    }
    candidates.push(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/local_llm").join(name),
    );
    candidates
}

pub(crate) fn resolve_python() -> Result<PathBuf, String> {
    if let Ok(p) = std::env::var("MALVIN_LOCAL_PYTHON") {
        let path = PathBuf::from(p);
        if path.is_file() || path.components().count() == 1 {
            return Ok(path);
        }
    }
    for name in ["python3", "python"] {
        if let Some(path) = crate::support_paths::lookup_bin_on_path(name) {
            return Ok(path);
        }
    }
    Err("python3 not found on PATH (required for local: models)".into())
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
            resolve_download_target("nemotron_cascade2")
                .expect("ok")
                .slug,
            "nemotron_cascade2"
        );
        assert!(resolve_download_target("openrouter:x").is_err());
        assert!(resolve_download_target("local:unknown").is_err());
    }

    #[test]
    fn local_llm_script_finds_repo_scripts() {
        let path = local_llm_script("server.py").expect("server.py in repo");
        assert!(path.ends_with("server.py"));
        assert!(path.is_file());
    }

    #[test]
    fn ensure_model_cached_errors_when_download_disabled() {
        let spec = require_known_local_slug("qwen35_9b_q4").expect("spec");
        if is_model_cached(spec) {
            return;
        }
        let err = ensure_model_cached(spec, false).expect_err("no download");
        assert!(err.contains("--no-download") || err.contains("not cached"));
    }

    #[test]
    fn resolve_python_finds_interpreter() {
        let py = resolve_python().expect("python3 on PATH in CI/dev");
        assert!(!py.as_os_str().is_empty());
    }
}
