use std::path::{Path, PathBuf};

const ENV_BRIDGE: &str = "MALVIN_CURSOR_SDK_BRIDGE";
const BRIDGE_JS: &str = "cursor-sdk-bridge/dist/bridge.js";
const SDK_MARKER: &str = "cursor-sdk-bridge/node_modules/@cursor/sdk/package.json";

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
    if let Some(path) = cursor_first_ready_bridge_js() {
        return Ok(path);
    }
    super::cursor_sdk::bridge_install::ensure_installed()
        .map_err(|e| super::cursor_sdk::bridge_install::install_failed_message(&e))
}

pub fn resolve_models_js() -> Result<PathBuf, String> {
    let models = resolve_bridge_js()?.with_file_name("models.js");
    if models.is_file() {
        return Ok(models);
    }
    Err(format!("{} not found", models.display()))
}

fn cursor_first_ready_bridge_js() -> Option<PathBuf> {
    cursor_candidate_roots()
        .into_iter()
        .filter(|root| cursor_sdk_marker_present(root))
        .map(|root| root.join(BRIDGE_JS))
        .find(|candidate| candidate.is_file())
}

pub(crate) fn cursor_sdk_marker_present(root: &Path) -> bool {
    root.join(SDK_MARKER).is_file()
}

fn cursor_candidate_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        roots.push(cwd);
    }
    roots.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")));
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        roots.push(dir.to_path_buf());
        if let Some(parent) = dir.parent() {
            roots.push(parent.to_path_buf());
        }
    }
    roots
}
