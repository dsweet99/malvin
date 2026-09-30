use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

const MIN_NODE: (u32, u32) = (22, 13);

pub fn resolve_node_bin() -> Result<PathBuf, String> {
    static CACHED: OnceLock<Result<PathBuf, String>> = OnceLock::new();
    CACHED.get_or_init(resolve_node_bin_uncached).clone()
}

fn resolve_node_bin_uncached() -> Result<PathBuf, String> {
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
    if let Some(path) = read_sticky_node_bin() {
        return Ok(path);
    }
    let mut tried = Vec::new();
    for candidate in node_candidates() {
        tried.push(candidate.display().to_string());
        if node_is_modern(&candidate) {
            write_sticky_node_bin(&candidate);
            return Ok(candidate);
        }
    }
    Err(format!(
        "Node >= {}.{} required for cursor-sdk-bridge; tried: {}",
        MIN_NODE.0,
        MIN_NODE.1,
        tried.join(", ")
    ))
}

fn sticky_node_bin_path() -> PathBuf {
    crate::user_home::user_home_dir()
        .join(crate::MALVIN_USER_HOME_DIR)
        .join("node_bin")
}

fn read_sticky_node_bin() -> Option<PathBuf> {
    let path = std::fs::read_to_string(sticky_node_bin_path()).ok()?;
    let path = PathBuf::from(path.trim());
    if path.is_file() && node_is_modern(&path) {
        Some(path)
    } else {
        None
    }
}

fn write_sticky_node_bin(path: &std::path::Path) {
    let sticky = sticky_node_bin_path();
    if let Some(parent) = sticky.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(&sticky, path.to_string_lossy().as_bytes());
}

fn node_candidates() -> Vec<PathBuf> {
    let mut out = Vec::new();
    push_unique(&mut out, crate::support_paths::lookup_bin_on_path("node"));
    if let Some(agent) = crate::support_paths::agent_or_cursor_agent_bin()
        && let Some(dir) = agent.parent()
    {
        push_unique(&mut out, Some(dir.join("node")));
    }
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
    dirs.sort_by(|a, b| b.cmp(a));
    dirs.into_iter()
        .map(|d| d.join("node"))
        .filter(|p| p.is_file())
        .collect()
}

fn node_is_modern(bin: &std::path::Path) -> bool {
    node_version(bin).is_some_and(|v| v >= MIN_NODE)
}

fn node_version(bin: &std::path::Path) -> Option<(u32, u32)> {
    let output = Command::new(bin).arg("--version").output().ok()?;
    if !output.status.success() {
        return None;
    }
    parse_node_version(&String::from_utf8_lossy(&output.stdout))
}

fn parse_node_version(text: &str) -> Option<(u32, u32)> {
    let mut parts = text.trim().trim_start_matches('v').split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    Some((major, minor))
}

pub(crate) fn apply_quiet_node_cli(cmd: &mut tokio::process::Command) {
    cmd.arg("--no-warnings");
    cmd.env("NODE_NO_WARNINGS", "1");
}

pub fn apply_quiet_node_cli_std(cmd: &mut std::process::Command) {
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
        assert!(node_version(&path).unwrap() >= MIN_NODE);
    }

    #[test]
    fn parse_node_version_rejects_old_minor() {
        assert_eq!(parse_node_version("v22.12.0\n"), Some((22, 12)));
        assert!(parse_node_version("v22.12.0").unwrap() < MIN_NODE);
        assert!(parse_node_version("v22.13.1").unwrap() >= MIN_NODE);
        assert!(parse_node_version("v24.0.0").unwrap() >= MIN_NODE);
        assert_eq!(parse_node_version("nope"), None);
    }

    #[test]
    fn cursor_agent_version_nodes_lists_files_when_present() {
        let found = cursor_agent_version_nodes();
        for p in &found {
            assert!(p.is_file(), "{}", p.display());
            assert_eq!(p.file_name().and_then(|s| s.to_str()), Some("node"));
        }
    }

    #[test]
    fn quiet_node_cli_suppresses_sqlite_experimental_warning() {
        let node = resolve_node_bin().expect("modern node");
        // Parent sandboxes often export NODE_NO_WARNINGS=1; clear it so the child
        // only stays quiet when apply_quiet_node_cli_std sets flags/env.
        let noisy = Command::new(&node)
            .env_remove("NODE_NO_WARNINGS")
            .args([
                "-e",
                r#"process.emitWarning("SQLite is an experimental feature","ExperimentalWarning")"#,
            ])
            .output()
            .expect("spawn noisy node");
        let noisy_stderr = String::from_utf8_lossy(&noisy.stderr);
        assert!(
            noisy_stderr.contains("ExperimentalWarning")
                && noisy_stderr.contains("SQLite is an experimental"),
            "precondition: unclean node should emit SQLite ExperimentalWarning, got: {noisy_stderr}"
        );

        let mut quiet = Command::new(&node);
        quiet.env_remove("NODE_NO_WARNINGS");
        apply_quiet_node_cli_std(&mut quiet);
        let output = quiet
            .args([
                "-e",
                r#"process.emitWarning("SQLite is an experimental feature","ExperimentalWarning")"#,
            ])
            .output()
            .expect("spawn quiet node");
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
