//! Resolve the Node bridge entry script.

use std::path::{Path, PathBuf};

const ENV_BRIDGE: &str = "MALVIN_PRIME_SDK_BRIDGE";
const BRIDGE_JS: &str = "prime-sdk-bridge/dist/bridge.js";
const MODELS_JS: &str = "prime-sdk-bridge/dist/models.js";
const SDK_MARKER: &str = "prime-sdk-bridge/node_modules/prime-agent/package.json";

/// Path to `prime-sdk-bridge/dist/bridge.js` (or env override).
///
/// Prefers roots that also have `prime-agent` installed (so a packaged `dist/`
/// without `node_modules` does not shadow `~/.malvin_home/sdk-bridges/`).
///
/// # Errors
///
/// Returns an error when no bridge entry can be found.
pub fn prime_resolve_bridge_js() -> Result<PathBuf, String> {
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
    if let Some(path) = prime_first_ready_bridge_js() {
        return Ok(path);
    }
    if let Some(path) = prime_first_any_bridge_js() {
        return Ok(path);
    }
    Err(
        "prime-sdk-bridge/dist/bridge.js not found (Prime SDK bridge). \
         Reinstall with `cargo install malvin` (build.rs installs prime-agent under \
         ~/.malvin_home/sdk-bridges/), or run `npm ci && npm run build` in prime-sdk-bridge/, \
         or set MALVIN_PRIME_SDK_BRIDGE"
            .to_string(),
    )
}

/// One-shot models listing script.
///
/// # Errors
///
/// Returns an error when the models entry is missing.
pub fn prime_resolve_models_js() -> Result<PathBuf, String> {
    if let Ok(bridge) = prime_resolve_bridge_js() {
        let models = bridge.with_file_name("models.js");
        if models.is_file() {
            return Ok(models);
        }
    }
    if let Some(path) = prime_first_ready_models_js() {
        return Ok(path);
    }
    if let Some(path) = prime_first_any_models_js() {
        return Ok(path);
    }
    Err("prime-sdk-bridge/dist/models.js not found".to_string())
}

fn prime_first_ready_bridge_js() -> Option<PathBuf> {
    for root in prime_candidate_roots() {
        if !prime_sdk_marker_present(&root) {
            continue;
        }
        let candidate = root.join(BRIDGE_JS);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn prime_first_any_bridge_js() -> Option<PathBuf> {
    for root in prime_candidate_roots() {
        let candidate = root.join(BRIDGE_JS);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn prime_first_ready_models_js() -> Option<PathBuf> {
    for root in prime_candidate_roots() {
        if !prime_sdk_marker_present(&root) {
            continue;
        }
        let candidate = root.join(MODELS_JS);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn prime_first_any_models_js() -> Option<PathBuf> {
    for root in prime_candidate_roots() {
        let candidate = root.join(MODELS_JS);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

/// True when `root` has a usable `prime-agent` install beside the bridge.
pub(crate) fn prime_sdk_marker_present(root: &Path) -> bool {
    root.join(SDK_MARKER).is_file()
}

fn prime_candidate_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        roots.push(cwd);
    }
    roots.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")));
    roots.push(
        crate::user_home::user_home_dir()
            .join(".malvin_home")
            .join("sdk-bridges"),
    );
    roots.push(crate::user_home::user_home_dir().join(".local/share/prime-agent-node"));
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
