use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::npm_bridge_install::NpmBridgePackage;

macro_rules! payload_file {
    ($rel:literal) => {
        (
            $rel,
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/claude-sdk-bridge/",
                $rel
            ))
            .as_slice(),
        )
    };
}

pub(crate) const PAYLOAD: &[(&str, &[u8])] = &[
    payload_file!("package.json"),
    payload_file!("package-lock.json"),
    payload_file!("dist/bridge.js"),
    payload_file!("dist/parent_death.js"),
    payload_file!("dist/protocol.js"),
    payload_file!("dist/usage.js"),
];

const PACKAGE: NpmBridgePackage = NpmBridgePackage {
    dir_name: "claude-sdk-bridge",
    sdk_marker: "node_modules/@anthropic-ai/claude-agent-sdk/package.json",
    npm_label: "Claude Agent SDK (@anthropic-ai/claude-agent-sdk)",
    payload: PAYLOAD,
};

#[must_use]
pub fn default_install_dir() -> PathBuf {
    PACKAGE.default_install_dir()
}

pub fn ensure_installed() -> Result<PathBuf, String> {
    static CACHED: OnceLock<Result<PathBuf, String>> = OnceLock::new();
    crate::npm_bridge_install::ensure_installed_cached(&PACKAGE, &CACHED)
}

pub fn install_into(dest: &Path) -> Result<PathBuf, String> {
    PACKAGE.install_into(dest)
}

pub fn install_failed_message(reason: &str) -> String {
    format!(
        "Claude Agent SDK bridge is not installed and could not be installed: {reason}\nclaude: models need Node.js >= 22.13 with npm. Install them and retry (or set MALVIN_CLAUDE_SDK_BRIDGE)."
    )
}
