//! Resolve a Node.js binary suitable for `@cursor/sdk` (Node ≥ 22.13).

use std::path::PathBuf;
use std::process::Command;

/// Prefer `MALVIN_NODE`, then PATH/`node` candidates with major version ≥ 22.
///
/// # Errors
///
/// Returns an error when no suitable Node is found.
pub fn resolve_node_bin() -> Result<PathBuf, String> {
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
    let mut tried = Vec::new();
    for candidate in node_candidates() {
        tried.push(candidate.display().to_string());
        if node_major_version(&candidate).is_some_and(|m| m >= 22) {
            return Ok(candidate);
        }
    }
    Err(format!(
        "Node >= 22.13 required for cursor-sdk-bridge; tried: {}",
        tried.join(", ")
    ))
}

fn node_candidates() -> Vec<PathBuf> {
    let mut out = Vec::new();
    push_unique(&mut out, crate::support_paths::lookup_bin_on_path("node"));
    // Cursor agent ships a modern Node next to its CLI (when the CLI is on PATH).
    if let Some(agent) = crate::support_paths::agent_or_cursor_agent_bin() {
        if let Some(dir) = agent.parent() {
            push_unique(&mut out, Some(dir.join("node")));
        }
    }
    // Sandboxed quality gates often strip cursor-agent from PATH; still find bundled Node.
    for bundled in cursor_agent_version_nodes() {
        push_unique(&mut out, Some(bundled));
    }
    out
}

fn push_unique(out: &mut Vec<PathBuf>, candidate: Option<PathBuf>) {
    let Some(path) = candidate.filter(|p| p.is_file()) else {
        return;
    };
    if !out.iter().any(|p| p == &path) {
        out.push(path);
    }
}

/// Newest-first Node binaries under `~/.local/share/cursor-agent/versions/*/node`.
fn cursor_agent_version_nodes() -> Vec<PathBuf> {
    let versions = crate::user_home::user_home_dir().join(".local/share/cursor-agent/versions");
    let Ok(entries) = std::fs::read_dir(&versions) else {
        return Vec::new();
    };
    let mut dirs: Vec<_> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    dirs.sort_by(|a, b| b.cmp(a)); // version dir names sort newest-first for current scheme
    dirs.into_iter()
        .map(|d| d.join("node"))
        .filter(|p| p.is_file())
        .collect()
}

fn node_major_version(bin: &std::path::Path) -> Option<u32> {
    let output = Command::new(bin).arg("--version").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let trimmed = text.trim().trim_start_matches('v');
    let major = trimmed.split('.').next()?;
    major.parse().ok()
}

/// Suppress Node process warnings (e.g. `SQLite` `ExperimentalWarning`) on stderr.
pub(crate) fn apply_quiet_node_cli(cmd: &mut tokio::process::Command) {
    cmd.arg("--no-warnings");
    cmd.env("NODE_NO_WARNINGS", "1");
}

/// Same quieting for [`std::process::Command`] (e.g. `malvin models`).
pub(crate) fn apply_quiet_node_cli_std(cmd: &mut std::process::Command) {
    cmd.arg("--no-warnings");
    cmd.env("NODE_NO_WARNINGS", "1");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_node_bin_finds_modern_node() {
        let path = resolve_node_bin().expect("modern node");
        assert!(path.is_file());
        assert!(node_major_version(&path).unwrap() >= 22);
    }

    #[test]
    fn cursor_agent_version_nodes_lists_files_when_present() {
        let found = cursor_agent_version_nodes();
        // Host may lack cursor-agent installs; only assert shape when nonempty.
        for p in &found {
            assert!(p.is_file(), "{}", p.display());
            assert_eq!(p.file_name().and_then(|s| s.to_str()), Some("node"));
        }
    }

    #[test]
    fn quiet_node_cli_suppresses_sqlite_experimental_warning() {
        let node = resolve_node_bin().expect("modern node");
        let bridge_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("cursor-sdk-bridge");
        let mut cmd = std::process::Command::new(&node);
        apply_quiet_node_cli_std(&mut cmd);
        let output = cmd
            .args([
                "-e",
                r#"
import { Agent } from "@cursor/sdk";
try {
  await Agent.create({
    apiKey: "sk-test-invalid",
    model: { id: "composer-2" },
    local: { cwd: "/tmp", settingSources: ["project"] },
  });
} catch {
  // expected
}
"#,
            ])
            .current_dir(&bridge_dir)
            .output()
            .expect("spawn node");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("ExperimentalWarning"),
            "stderr should not contain ExperimentalWarning, got: {stderr}"
        );
        assert!(
            !stderr.contains("SQLite is an experimental"),
            "stderr should not mention SQLite warning, got: {stderr}"
        );
    }
}
