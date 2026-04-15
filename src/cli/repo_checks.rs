use std::path::Path;
use std::process::Command;

use super::kiss_clamp;
use malvin::output::{MALVIN_WHO, print_stdout_line};

pub fn run_repo_workspace_gates(work_dir: &Path) -> Result<(), String> {
    kiss_clamp::ensure_kiss_clamp_if_needed(work_dir)?;
    warn_kissconfig_test_coverage_if_needed(work_dir);
    run_pre_commit_checks_or_warn(work_dir)
}

pub fn warn_kissconfig_test_coverage_if_needed(work_dir: &Path) {
    let path = work_dir.join(".kissconfig");
    if !path.is_file() {
        return;
    }
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => {
            print_stdout_line(
                MALVIN_WHO,
                &format!("Warning: could not read .kissconfig: {e}"),
            );
            return;
        }
    };
    let value = match text.parse::<toml::Value>() {
        Ok(v) => v,
        Err(e) => {
            print_stdout_line(
                MALVIN_WHO,
                &format!("Warning: could not parse .kissconfig as TOML: {e}"),
            );
            return;
        }
    };
    if !should_warn_low_test_coverage(&value) {
        return;
    }
    print_stdout_line(
        MALVIN_WHO,
        "Warning: .kissconfig gate.test_coverage_threshold is missing or below 90; editing code without sufficient unit test coverage is dangerous.",
    );
}

fn trim_detail_chars(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        return s.to_string();
    }
    s.chars().take(max_chars).collect::<String>() + "…"
}

fn format_pre_commit_failure(output: &std::process::Output) -> String {
    let exit = output
        .status
        .code()
        .map_or_else(|| "signal".to_string(), |c| c.to_string());
    let out = String::from_utf8_lossy(&output.stdout);
    let err = String::from_utf8_lossy(&output.stderr);
    let mut parts: Vec<String> = Vec::new();
    if !out.trim().is_empty() {
        parts.push(format!("stdout:\n{out}"));
    }
    if !err.trim().is_empty() {
        parts.push(format!("stderr:\n{err}"));
    }
    let merged = if parts.is_empty() {
        "(no output)".to_string()
    } else {
        parts.join("\n")
    };
    let detail = trim_detail_chars(&merged, 4000);
    format!("`pre-commit run --all-files` failed (exit {exit}): {detail}")
}

fn should_warn_low_test_coverage(value: &toml::Value) -> bool {
    value
        .get("gate")
        .and_then(|g| g.get("test_coverage_threshold"))
        .and_then(toml::Value::as_integer)
        .is_none_or(|t| t < 90)
}

pub fn run_pre_commit_checks_or_warn(work_dir: &Path) -> Result<(), String> {
    let config = work_dir.join(".pre-commit-config.yaml");
    if !config.is_file() {
        print_stdout_line(
            MALVIN_WHO,
            "Warning: no .pre-commit-config.yaml; editing code without configured linters is risky.",
        );
        return Ok(());
    }
    print_stdout_line(
        MALVIN_WHO,
        "Running `pre-commit run --all-files` (repo-configured hooks)",
    );
    let output = Command::new("pre-commit")
        .args(["run", "--all-files"])
        .current_dir(work_dir)
        .output()
        .map_err(|e| format!("`pre-commit` failed to start: {e}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format_pre_commit_failure(&output))
    }
}

#[cfg(test)]
mod tests {
    use super::{format_pre_commit_failure, should_warn_low_test_coverage};

    #[test]
    fn pre_commit_failure_includes_exit_and_streams() {
        let out = std::process::Command::new("sh")
            .args(["-c", "echo out; echo err >&2; exit 7"])
            .output()
            .expect("sh");
        let msg = format_pre_commit_failure(&out);
        assert!(msg.contains("exit 7"), "{msg}");
        assert!(msg.contains("out"), "{msg}");
        assert!(msg.contains("err"), "{msg}");
    }

    #[test]
    fn coverage_warn_when_gate_missing() {
        let v: toml::Value = toml::from_str("").unwrap();
        assert!(should_warn_low_test_coverage(&v));
    }

    #[test]
    fn coverage_warn_when_threshold_missing() {
        let v: toml::Value = toml::from_str("[gate]\nmin_similarity = 0.7\n").unwrap();
        assert!(should_warn_low_test_coverage(&v));
    }

    #[test]
    fn coverage_warn_when_threshold_below_90() {
        let v: toml::Value = toml::from_str("[gate]\ntest_coverage_threshold = 89\n").unwrap();
        assert!(should_warn_low_test_coverage(&v));
    }

    #[test]
    fn coverage_ok_at_90() {
        let v: toml::Value = toml::from_str("[gate]\ntest_coverage_threshold = 90\n").unwrap();
        assert!(!should_warn_low_test_coverage(&v));
    }

    #[test]
    fn coverage_ok_above_90() {
        let v: toml::Value = toml::from_str("[gate]\ntest_coverage_threshold = 100\n").unwrap();
        assert!(!should_warn_low_test_coverage(&v));
    }
}
