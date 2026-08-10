//! Resolve the Node bridge entry script.

use std::path::PathBuf;

const ENV_BRIDGE: &str = "MALVIN_CURSOR_SDK_BRIDGE";

/// Path to `cursor-sdk-bridge/dist/bridge.js` (or env override).
///
/// # Errors
///
/// Returns an error when no bridge entry can be found.
pub fn resolve_bridge_js() -> Result<PathBuf, String> {
    if let Some(p) = std::env::var_os(ENV_BRIDGE).filter(|v| !v.is_empty()) {
        let path = PathBuf::from(p);
        if path.is_file() {
            return Ok(path);
        }
        return Err(format!(
            "{ENV_BRIDGE} is set but not a file: {}",
            path.display()
        ));
    }
    for root in candidate_roots() {
        let candidate = root.join("cursor-sdk-bridge/dist/bridge.js");
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    Err(
        "cursor-sdk-bridge/dist/bridge.js not found; run `npm ci && npm run build` in cursor-sdk-bridge/ or set MALVIN_CURSOR_SDK_BRIDGE"
            .to_string(),
    )
}

/// One-shot models listing script.
///
/// # Errors
///
/// Returns an error when the models entry is missing.
pub fn resolve_models_js() -> Result<PathBuf, String> {
    if let Ok(bridge) = resolve_bridge_js() {
        let models = bridge.with_file_name("models.js");
        if models.is_file() {
            return Ok(models);
        }
    }
    for root in candidate_roots() {
        let candidate = root.join("cursor-sdk-bridge/dist/models.js");
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    Err("cursor-sdk-bridge/dist/models.js not found".to_string())
}

fn candidate_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        roots.push(cwd);
    }
    roots.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")));
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            roots.push(dir.to_path_buf());
            if let Some(parent) = dir.parent() {
                roots.push(parent.to_path_buf());
            }
        }
    }
    roots
}
