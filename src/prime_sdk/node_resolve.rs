//! Resolve a Node.js binary suitable for `prime-agent` (Node ≥ 22.8).
//!
//! Uses a separate sticky file from Cursor (`node_bin_prime`) so a Node that
//! satisfies Prime's floor is never persisted for Cursor's higher floor.

use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

const MIN_MAJOR: u32 = 22;
const MIN_MINOR: u32 = 8;

/// Prefer `MALVIN_NODE`, then PATH / prime-agent bundled Node with version ≥ 22.8.
///
/// # Errors
///
/// Returns an error when no suitable Node is found.
pub fn prime_resolve_node_bin() -> Result<PathBuf, String> {
    static CACHED: OnceLock<Result<PathBuf, String>> = OnceLock::new();
    CACHED.get_or_init(prime_resolve_node_bin_uncached).clone()
}

fn prime_resolve_node_bin_uncached() -> Result<PathBuf, String> {
    if let Some(p) = std::env::var_os("MALVIN_NODE").filter(|v| !v.is_empty()) {
        let path = PathBuf::from(p);
        if path.is_file() {
            return Ok(path);
        }
        return Err(format!(
            "MALVIN_NODE is set but not a file: {}",
            path.display()
        ));
    }
    if let Some(path) = prime_read_sticky_node_bin() {
        return Ok(path);
    }
    let mut tried = Vec::new();
    for candidate in prime_node_candidates() {
        tried.push(candidate.display().to_string());
        if prime_node_meets_floor(&candidate) {
            prime_write_sticky_node_bin(&candidate);
            return Ok(candidate);
        }
    }
    Err(format!(
        "Node >= {MIN_MAJOR}.{MIN_MINOR} required for prime-sdk-bridge; tried: {}",
        tried.join(", ")
    ))
}

fn prime_sticky_node_bin_path() -> PathBuf {
    crate::user_home::user_home_dir()
        .join(".malvin_home")
        .join("node_bin_prime")
}

fn prime_read_sticky_node_bin() -> Option<PathBuf> {
    let path = std::fs::read_to_string(prime_sticky_node_bin_path()).ok()?;
    let path = PathBuf::from(path.trim());
    if path.is_file() && prime_node_meets_floor(&path) {
        Some(path)
    } else {
        None
    }
}

fn prime_write_sticky_node_bin(path: &std::path::Path) {
    let sticky = prime_sticky_node_bin_path();
    if let Some(parent) = sticky.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(&sticky, path.to_string_lossy().as_bytes());
}

fn prime_node_candidates() -> Vec<PathBuf> {
    let mut out = Vec::new();
    prime_push_unique(&mut out, crate::support_paths::lookup_bin_on_path("node"));
    // Prime-agent install ships Node under ~/.local/share/prime-agent-node/...
    for bundled in prime_agent_nodes() {
        prime_push_unique(&mut out, Some(bundled));
    }
    out
}

fn prime_push_unique(out: &mut Vec<PathBuf>, candidate: Option<PathBuf>) {
    let Some(path) = candidate.filter(|p| p.is_file()) else {
        return;
    };
    if !out.iter().any(|p| p == &path) {
        out.push(path);
    }
}

fn prime_agent_nodes() -> Vec<PathBuf> {
    let root = crate::user_home::user_home_dir().join(".local/share/prime-agent-node");
    let mut out = Vec::new();
    let current = root.join("current/bin/node");
    if current.is_file() {
        out.push(current);
    }
    if let Ok(entries) = std::fs::read_dir(&root) {
        let mut dirs: Vec<_> = entries
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .collect();
        dirs.sort_by(|a, b| b.cmp(a));
        for d in dirs {
            let node = d.join("bin/node");
            if node.is_file() {
                out.push(node);
            }
        }
    }
    out
}

fn prime_node_meets_floor(bin: &std::path::Path) -> bool {
    let Some((major, minor)) = prime_node_major_minor(bin) else {
        return false;
    };
    major > MIN_MAJOR || (major == MIN_MAJOR && minor >= MIN_MINOR)
}

fn prime_node_major_minor(bin: &std::path::Path) -> Option<(u32, u32)> {
    let output = Command::new(bin).arg("--version").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let trimmed = text.trim().trim_start_matches('v');
    let mut parts = trimmed.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    Some((major, minor))
}

pub(crate) fn prime_apply_quiet_node_cli(cmd: &mut tokio::process::Command) {
    cmd.arg("--no-warnings");
    cmd.env("NODE_NO_WARNINGS", "1");
}

pub(crate) fn prime_apply_quiet_node_cli_std(cmd: &mut std::process::Command) {
    cmd.arg("--no-warnings");
    cmd.env("NODE_NO_WARNINGS", "1");
}
