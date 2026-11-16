use std::path::PathBuf;

const ENV_BRIDGE: &str = "MALVIN_CLAUDE_SDK_BRIDGE";
const BRIDGE_JS: &str = "bridges/claude-sdk-bridge/dist/bridge.js";
const SDK_MARKER: &str =
    "bridges/claude-sdk-bridge/node_modules/@anthropic-ai/claude-agent-sdk/package.json";

pub fn resolve_bridge_js() -> Result<PathBuf, String> {
    if let Some(result) = crate::workspace::support_paths::file_from_env_override(ENV_BRIDGE) {
        return result;
    }
    for root in candidate_roots() {
        let bridge = root.join(BRIDGE_JS);
        if root.join(SDK_MARKER).is_file() && bridge.is_file() {
            return Ok(bridge);
        }
    }
    super::claude_sdk::bridge_install::ensure_installed()
        .map_err(|error| super::claude_sdk::bridge_install::install_failed_message(&error))
}

fn candidate_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        roots.push(cwd);
    }
    roots.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")));
    roots
}
