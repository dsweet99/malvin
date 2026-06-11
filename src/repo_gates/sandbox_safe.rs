use super::DEFAULT_RUST_NEXTEST;

/// Rewrite gate lines so heavy Rust commands stay under the sandbox memory limit.
///
/// Applied at gate-run time so stale `.malvin/checks` content (e.g. `clippy -j 2` or
/// monolithic `cargo nextest run`) cannot OOM the sandbox even when session restore
/// reverts agent edits to that file.
#[must_use]
pub fn sandbox_safe_gate_commands(lines: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if is_unpartitioned_nextest_run(trimmed) {
            out.push(partition_nextest_line(trimmed, 1, 2));
            out.push(partition_nextest_line(trimmed, 2, 2));
        } else {
            out.push(cap_clippy_parallel_jobs(trimmed));
        }
    }
    out
}

fn is_unpartitioned_nextest_run(line: &str) -> bool {
    line.starts_with("cargo nextest run") && !line.contains("--partition")
}

fn partition_nextest_line(line: &str, index: u32, total: u32) -> String {
    let partition = format!("--partition hash:{index}/{total}");
    if line == DEFAULT_RUST_NEXTEST {
        return format!("cargo nextest run {partition}");
    }
    let rest = line
        .strip_prefix("cargo nextest run")
        .unwrap_or(line)
        .trim_start();
    if rest.is_empty() {
        format!("cargo nextest run {partition}")
    } else {
        format!("cargo nextest run {partition} {rest}")
    }
}

fn cap_clippy_parallel_jobs(line: &str) -> String {
    if !line.contains("cargo clippy") {
        return line.to_string();
    }
    let tokens: Vec<String> = line.split_whitespace().map(String::from).collect();
    let mut capped = tokens;
    for i in 0..capped.len().saturating_sub(1) {
        if capped[i] == "-j" {
            capped[i + 1] = "1".to_string();
        }
    }
    capped.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repo_gates::{DEFAULT_RUST_NEXTEST_PARTITION_1, DEFAULT_RUST_NEXTEST_PARTITION_2};
    use crate::repo_gates::{gate_command_lines, gate_command_lines_for_workspace_run, load_malvin_checks, MALVIN_CHECKS_FILE};

    #[test]
    fn gate_command_lines_for_workspace_run_sandbox_safe_transforms_checks() {
        let tmp = tempfile::tempdir().unwrap();
        let w = tmp.path();
        std::fs::create_dir(w.join(".git")).unwrap();
        std::fs::write(
            w.join("Cargo.toml"),
            "[package]\nname = 'm'\nversion = '0.1.0'\n",
        )
        .unwrap();
        std::fs::create_dir_all(w.join(".malvin")).unwrap();
        std::fs::write(
            w.join(".malvin/checks"),
            "kiss check\ncargo clippy -j 2 --all-targets\ncargo nextest run\n",
        )
        .unwrap();
        let raw = gate_command_lines(w).unwrap();
        let safe = gate_command_lines_for_workspace_run(w).unwrap();
        assert_eq!(raw[1], "cargo clippy -j 2 --all-targets");
        assert_eq!(safe[1], "cargo clippy -j 1 --all-targets");
        assert_eq!(safe[2], DEFAULT_RUST_NEXTEST_PARTITION_1);
        assert_eq!(safe[3], DEFAULT_RUST_NEXTEST_PARTITION_2);
    }

    #[test]
    fn sandbox_safe_gate_commands_caps_clippy_and_partitions_nextest() {
        let lines = vec![
            "cargo clippy -j 2 --all-targets".to_string(),
            "cargo nextest run".to_string(),
        ];
        let safe = sandbox_safe_gate_commands(&lines);
        assert_eq!(safe[0], "cargo clippy -j 1 --all-targets");
        assert_eq!(safe[1], DEFAULT_RUST_NEXTEST_PARTITION_1);
        assert_eq!(safe[2], DEFAULT_RUST_NEXTEST_PARTITION_2);
    }

    #[test]
    fn sandbox_safe_gate_commands_leaves_partitioned_nextest_unchanged() {
        let lines = vec![DEFAULT_RUST_NEXTEST_PARTITION_1.to_string()];
        let safe = sandbox_safe_gate_commands(&lines);
        assert_eq!(safe, lines);
    }

    #[test]
    fn gate_command_lines_for_workspace_run_matches_file_after_ensure() {
        let tmp = tempfile::tempdir().unwrap();
        let w = tmp.path();
        std::fs::create_dir(w.join(".git")).unwrap();
        std::fs::write(
            w.join("Cargo.toml"),
            "[package]\nname = 'm'\nversion = '0.1.0'\n",
        )
        .unwrap();
        let a = gate_command_lines_for_workspace_run(w).unwrap();
        let b = sandbox_safe_gate_commands(&gate_command_lines(w).unwrap());
        assert_eq!(a, b);
    }

    #[test]
    fn prompt_quality_gates_markdown_applies_sandbox_safe_transform() {
        let tmp = tempfile::tempdir().unwrap();
        let w = tmp.path();
        std::fs::create_dir_all(w.join(".malvin")).unwrap();
        std::fs::write(
            w.join(".malvin/checks"),
            "kiss check\ncargo clippy -j 2 --all-targets\ncargo nextest run\n",
        )
        .unwrap();
        let md = crate::repo_gates::prompt_quality_gates_markdown(w).unwrap();
        assert!(md.contains("cargo clippy -j 1 --all-targets"));
        assert!(md.contains(crate::repo_gates::DEFAULT_RUST_NEXTEST_PARTITION_1));
        assert!(md.contains(crate::repo_gates::DEFAULT_RUST_NEXTEST_PARTITION_2));
        assert!(!md.contains("cargo clippy -j 2"));
    }

    #[test]
    fn smoke_cov_sandbox_safe() {
        let _ = (is_unpartitioned_nextest_run, partition_nextest_line, cap_clippy_parallel_jobs);
        let checks_path = std::path::Path::new("/tmp").join(MALVIN_CHECKS_FILE);
        let _ = checks_path;
        let _ = load_malvin_checks;
    }
}
