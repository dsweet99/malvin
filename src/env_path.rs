//! Resolve executable names on `PATH` (shared by ACP spawn resolution and CLI helpers).

use std::path::PathBuf;

/// Returns an absolute path to `bin` if it exists as a regular file on `PATH`.
#[must_use]
pub fn lookup_bin_on_path(bin: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|dir| dir.join(bin))
        .find(|candidate| candidate.is_file())
}

/// Prefer `agent`, then `cursor-agent` (same default order as ACP spawn when `MALVIN_AGENT_ACP_BIN` is unset).
#[must_use]
pub fn agent_or_cursor_agent_bin() -> Option<PathBuf> {
    lookup_bin_on_path("agent").or_else(|| lookup_bin_on_path("cursor-agent"))
}
