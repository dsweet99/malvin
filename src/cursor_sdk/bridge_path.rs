use std::path::{Path, PathBuf};

const ENV_BRIDGE: &str = "MALVIN_CURSOR_SDK_BRIDGE";
const BRIDGE_JS: &str = "cursor-sdk-bridge/dist/bridge.js";
const MODELS_JS: &str = "cursor-sdk-bridge/dist/models.js";
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
    if let Some(path) = cursor_first_any_bridge_js() {
        return Ok(path);
    }
    Err(
        "cursor-sdk-bridge/dist/bridge.js not found (Cursor SDK bridge). \
         Reinstall with `cargo install malvin` (build.rs installs @cursor/sdk under \
         ~/.malvin_home/sdk-bridges/), or run `npm ci && npm run build` in cursor-sdk-bridge/, \
         or set MALVIN_CURSOR_SDK_BRIDGE"
            .to_string(),
    )
}

pub fn resolve_models_js() -> Result<PathBuf, String> {
    if let Ok(bridge) = resolve_bridge_js() {
        let models = bridge.with_file_name("models.js");
        if models.is_file() {
            return Ok(models);
        }
    }
    if let Some(path) = cursor_first_ready_models_js() {
        return Ok(path);
    }
    if let Some(path) = cursor_first_any_models_js() {
        return Ok(path);
    }
    Err("cursor-sdk-bridge/dist/models.js not found".to_string())
}

fn cursor_first_ready_bridge_js() -> Option<PathBuf> {
    for root in cursor_candidate_roots() {
        if !cursor_sdk_marker_present(&root) {
            continue;
        }
        let candidate = root.join(BRIDGE_JS);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn cursor_first_any_bridge_js() -> Option<PathBuf> {
    for root in cursor_candidate_roots() {
        let candidate = root.join(BRIDGE_JS);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn cursor_first_ready_models_js() -> Option<PathBuf> {
    for root in cursor_candidate_roots() {
        if !cursor_sdk_marker_present(&root) {
            continue;
        }
        let candidate = root.join(MODELS_JS);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn cursor_first_any_models_js() -> Option<PathBuf> {
    for root in cursor_candidate_roots() {
        let candidate = root.join(MODELS_JS);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
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
    roots.push(
        crate::user_home::user_home_dir()
            .join(".malvin_home")
            .join("sdk-bridges"),
    );
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
