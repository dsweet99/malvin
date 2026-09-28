use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use super::bridge_install_npm::npm_ci;

pub const SETUP_COMMAND: &str = "malvin admin setup-cursor";
const DIR_NAME: &str = "cursor-sdk-bridge";
const SDK_MARKER: &str = "node_modules/@cursor/sdk/package.json";
const LOCK_JSON: &str = "package-lock.json";
const NPM_STAMP: &str = ".malvin-npm-stamp";
const INSTALL_LOCK: &str = ".malvin-install.lock";

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

pub(crate) fn fnv1a64(data: &[u8]) -> u64 {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0100_0000_01b3;
    let mut hash = OFFSET;
    for byte in data {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

#[must_use]
pub fn default_install_dir() -> PathBuf {
    crate::user_home::user_home_dir()
        .join(".malvin_home")
        .join("sdk-bridges")
        .join(DIR_NAME)
}

pub fn ensure_installed() -> Result<PathBuf, String> {
    static CACHED: OnceLock<Result<PathBuf, String>> = OnceLock::new();
    CACHED
        .get_or_init(|| install_into(&default_install_dir()))
        .clone()
}

pub fn install_into(dest: &Path) -> Result<PathBuf, String> {
    fs::create_dir_all(dest).map_err(|e| format!("mkdir {}: {e}", dest.display()))?;
    let _lock = lock_install_dir(dest)?;
    write_payload(dest)?;
    if !npm_deps_current(dest) {
        npm_ci(dest)?;
        verify_sdk_installed(dest)?;
        write_npm_stamp(dest)?;
    }
    Ok(dest.join("dist").join("bridge.js"))
}

fn lock_install_dir(dest: &Path) -> Result<fs::File, String> {
    let path = dest.join(INSTALL_LOCK);
    let file = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(&path)
        .map_err(|e| format!("open {}: {e}", path.display()))?;
    file.lock()
        .map_err(|e| format!("lock {}: {e}", path.display()))?;
    Ok(file)
}

fn write_payload(dest: &Path) -> Result<(), String> {
    for (rel, bytes) in PAYLOAD {
        let path = dest.join(rel);
        if fs::read(&path).is_ok_and(|on_disk| on_disk == *bytes) {
            continue;
        }
        write_atomically(&path, bytes)?;
    }
    Ok(())
}

fn write_atomically(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("mkdir {}: {e}", parent.display()))?;
    }
    let tmp = path.with_extension("malvin-tmp");
    fs::write(&tmp, bytes).map_err(|e| format!("write {}: {e}", tmp.display()))?;
    fs::rename(&tmp, path).map_err(|e| format!("rename {}: {e}", path.display()))
}

fn lock_stamp() -> String {
    let lock = PAYLOAD
        .iter()
        .find(|(rel, _)| *rel == LOCK_JSON)
        .map_or(&[][..], |(_, bytes)| *bytes);
    format!("{:x}", fnv1a64(lock))
}

pub(crate) fn npm_deps_current(dest: &Path) -> bool {
    dest.join(SDK_MARKER).is_file()
        && fs::read_to_string(dest.join(NPM_STAMP)).unwrap_or_default().trim() == lock_stamp()
}

fn verify_sdk_installed(dest: &Path) -> Result<(), String> {
    if dest.join(SDK_MARKER).is_file() {
        return Ok(());
    }
    Err(format!(
        "npm ci finished but {} is missing",
        dest.join(SDK_MARKER).display()
    ))
}

fn write_npm_stamp(dest: &Path) -> Result<(), String> {
    let path = dest.join(NPM_STAMP);
    fs::write(&path, format!("{}\n", lock_stamp()))
        .map_err(|e| format!("write {}: {e}", path.display()))
}

#[must_use]
pub fn install_failed_message(reason: &str) -> String {
    format!(
        "Cursor SDK bridge is not installed and could not be installed: {reason}\n\
         cursor: models need Node.js >= 22.13 with npm. Install them, then run \
         `{SETUP_COMMAND}` (or set MALVIN_CURSOR_SDK_BRIDGE). \
         rpi: and codex: models do not need Node."
    )
}
