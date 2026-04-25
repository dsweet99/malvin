use std::path::Path;
use std::process::Command;
use super::kiss_clamp;
use malvin::output::{MALVIN_WHO, print_stdout_line};

#[derive(Clone, Copy)]
pub enum RepoGateOutput {
    Tagged,
    Plain,
}

pub fn emit_repo_gate_stdout_line(output: RepoGateOutput, line: &str) {
    match output {
        RepoGateOutput::Tagged => print_stdout_line(MALVIN_WHO, line),
        RepoGateOutput::Plain => println!("{line}"),
    }
}

pub fn run_repo_workspace_gates(work_dir: &Path, output: RepoGateOutput) -> Result<(), String> {
    ensure_workspace_style_markers(work_dir, output)?;
    kiss_clamp::ensure_kiss_clamp_if_needed(work_dir, output)?;
    warn_kissconfig_test_coverage_if_needed(work_dir, output);
    run_pre_commit_checks_or_warn(work_dir, output)
}

/// Touch `<work_dir>/grounding.md` and `<work_dir>/.llm_style/style.md` when missing.
/// Existing files are never touched.
/// Returns an error string if a file or the `.llm_style` directory cannot be created.
pub fn ensure_workspace_style_markers(work_dir: &Path, output: RepoGateOutput) -> Result<(), String> {
    touch_if_missing(&work_dir.join("grounding.md"), output)?;
    let style_dir = work_dir.join(".llm_style");
    if !style_dir.is_dir() {
        std::fs::create_dir_all(&style_dir)
            .map_err(|e| format!("create {}: {e}", style_dir.display()))?;
    }
    touch_if_missing(&style_dir.join("style.md"), output)
}

fn touch_if_missing(path: &Path, output: RepoGateOutput) -> Result<(), String> {
    if path.exists() {
        if path.is_file() {
            return Ok(());
        }
        return Err(format!("{} exists but is not a file", path.display()));
    }
    std::fs::File::create(path).map_err(|e| format!("create {}: {e}", path.display()))?;
    emit_repo_gate_stdout_line(output, &format!("Touched empty {} (was missing)", path.display()));
    Ok(())
}

pub fn warn_kissconfig_test_coverage_if_needed(work_dir: &Path, output: RepoGateOutput) {
    let path = work_dir.join(".kissconfig");
    if !path.is_file() {
        return;
    }
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => {
            emit_repo_gate_stdout_line(output, &format!("Warning: could not read .kissconfig: {e}"));
            return;
        }
    };
    let value = match text.parse::<toml::Value>() {
        Ok(v) => v,
        Err(e) => {
            emit_repo_gate_stdout_line(
                output,
                &format!("Warning: could not parse .kissconfig as TOML: {e}"),
            );
            return;
        }
    };
    if !should_warn_low_test_coverage(&value) {
        return;
    }
    emit_repo_gate_stdout_line(
        output,
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

pub fn run_pre_commit_checks_or_warn(work_dir: &Path, output: RepoGateOutput) -> Result<(), String> {
    let config = work_dir.join(".pre-commit-config.yaml");
    if !config.is_file() {
        emit_repo_gate_stdout_line(
            output,
            "Warning: no .pre-commit-config.yaml; editing code without configured linters is risky.",
        );
        return Ok(());
    }
    emit_repo_gate_stdout_line(output, "Running `pre-commit run --all-files` (repo-configured hooks)");
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
    use super::{
        RepoGateOutput, ensure_workspace_style_markers, format_pre_commit_failure,
        should_warn_low_test_coverage,
    };

    #[test]
    fn pre_commit_failure_includes_exit_and_streams() {
        let _ = (
            stringify!(super::emit_repo_gate_stdout_line),
            stringify!(super::touch_if_missing),
            stringify!(super::trim_detail_chars),
        );
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

    #[test]
    fn style_markers_are_touched_when_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let work = tmp.path();
        ensure_workspace_style_markers(work, RepoGateOutput::Tagged).unwrap();
        let grounding = work.join("grounding.md");
        let style = work.join(".llm_style").join("style.md");
        assert!(grounding.is_file(), "grounding.md not created");
        assert!(style.is_file(), "style.md not created");
        assert_eq!(std::fs::read(&grounding).unwrap(), Vec::<u8>::new());
        assert_eq!(std::fs::read(&style).unwrap(), Vec::<u8>::new());
    }

    #[test]
    fn style_markers_preserve_existing_content() {
        let tmp = tempfile::tempdir().unwrap();
        let work = tmp.path();
        std::fs::create_dir_all(work.join(".llm_style")).unwrap();
        std::fs::write(work.join("grounding.md"), b"KEEP ME\n").unwrap();
        std::fs::write(work.join(".llm_style").join("style.md"), b"STYLE STAYS\n").unwrap();
        ensure_workspace_style_markers(work, RepoGateOutput::Tagged).unwrap();
        assert_eq!(
            std::fs::read_to_string(work.join("grounding.md")).unwrap(),
            "KEEP ME\n"
        );
        assert_eq!(
            std::fs::read_to_string(work.join(".llm_style").join("style.md")).unwrap(),
            "STYLE STAYS\n"
        );
    }

    #[test]
    fn style_markers_mixed_touch_only_missing_one() {
        let tmp = tempfile::tempdir().unwrap();
        let work = tmp.path();
        std::fs::write(work.join("grounding.md"), b"ORIGINAL\n").unwrap();
        ensure_workspace_style_markers(work, RepoGateOutput::Plain).unwrap();
        assert_eq!(
            std::fs::read_to_string(work.join("grounding.md")).unwrap(),
            "ORIGINAL\n"
        );
        let style = work.join(".llm_style").join("style.md");
        assert!(style.is_file());
        assert_eq!(std::fs::read(&style).unwrap(), Vec::<u8>::new());
    }

    #[test]
    fn style_markers_error_when_grounding_path_is_directory() {
        let tmp = tempfile::tempdir().unwrap();
        let work = tmp.path();
        std::fs::create_dir(work.join("grounding.md")).unwrap();
        assert!(
            ensure_workspace_style_markers(work, RepoGateOutput::Plain)
                .unwrap_err()
                .contains("exists but is not a file")
        );
    }
}
