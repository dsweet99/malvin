use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::npm_bridge_install::NpmBridgePackage;
#[cfg(test)]
pub(crate) use crate::npm_bridge_install::fnv1a64;

macro_rules! payload_file {
    ($rel:literal) => {
        (
            $rel,
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/cursor-sdk-bridge/",
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
    payload_file!("dist/bridge_policy.js"),
    payload_file!("dist/model_selection.js"),
    payload_file!("dist/models.js"),
    payload_file!("dist/parent_death.js"),
    payload_file!("dist/protocol.js"),
    payload_file!("dist/sdk_map.js"),
];

const PACKAGE: NpmBridgePackage = NpmBridgePackage {
    dir_name: "cursor-sdk-bridge",
    sdk_marker: "node_modules/@cursor/sdk/package.json",
    npm_label: "Cursor SDK (@cursor/sdk)",
    payload: PAYLOAD,
};

#[must_use]
pub fn default_install_dir() -> PathBuf {
    PACKAGE.default_install_dir()
}

pub fn ensure_installed() -> Result<PathBuf, String> {
    static CACHED: OnceLock<Result<PathBuf, String>> = OnceLock::new();
    CACHED
        .get_or_init(|| install_into(&default_install_dir()))
        .clone()
}

pub fn install_into(dest: &Path) -> Result<PathBuf, String> {
    PACKAGE.install_into(dest)
}

#[cfg(test)]
pub(crate) fn npm_deps_current(dest: &Path) -> bool {
    PACKAGE.npm_deps_current(dest)
}

#[must_use]
pub fn install_failed_message(reason: &str) -> String {
    format!(
        "Cursor SDK bridge is not installed and could not be installed: {reason}\n\
         cursor: models need Node.js >= 22.13 with npm. Install them and retry \
         (or set MALVIN_CURSOR_SDK_BRIDGE). \
         codex: models do not need Node."
    )
}
