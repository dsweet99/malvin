//! When `malvin init` should run agent-based `.malvin/checks` discovery.

use std::path::Path;
use std::process::Command;

use super::discover_py::python_ruff_and_pytest_flags;

/// Whether the repo has at least one git commit.
#[must_use]
pub fn repo_has_git_commits(root: &Path) -> bool {
    Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(root)
        .output()
        .is_ok_and(|o| o.status.success())
}

/// Same signals as [`super::builtin_gate_command_lines`] for "meaningful" tooling.
#[must_use]
pub fn repo_has_meaningful_source_or_tooling(root: &Path) -> bool {
    let (has_py, has_pytest) = python_ruff_and_pytest_flags(root);
    has_py
        || has_pytest
        || root.join("Cargo.toml").is_file()
        || root.join("pyproject.toml").is_file()
        || root.join("tests").is_dir()
        || root.join(".pre-commit-config.yaml").is_file()
}

/// Skip discovery when there are no commits and no meaningful source/tooling artifacts.
#[must_use]
pub fn init_repo_is_empty_for_discovery(root: &Path) -> bool {
    !repo_has_git_commits(root) && !repo_has_meaningful_source_or_tooling(root)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InitDiscoveryDecision {
    pub run: bool,
    pub skip_reason: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InitDiscoveryRequest {
    pub checks_existed_before_bootstrap: bool,
    pub force_overwrite: bool,
}

/// Whether init should run the `KPop` checks-discovery phase after bootstrap.
#[must_use]
pub fn init_discovery_decision(
    root: &Path,
    request: InitDiscoveryRequest,
) -> InitDiscoveryDecision {
    if init_repo_is_empty_for_discovery(root) {
        return InitDiscoveryDecision {
            run: false,
            skip_reason: Some("empty repo; using builtin checks"),
        };
    }
    if request.checks_existed_before_bootstrap && !request.force_overwrite {
        return InitDiscoveryDecision {
            run: false,
            skip_reason: Some("checks already present; discovery skipped"),
        };
    }
    InitDiscoveryDecision {
        run: true,
        skip_reason: None,
    }
}

/// Structural validation for a proposed `.malvin/checks` file (no full gate run).
pub fn validate_checks_command_lines(lines: &[String]) -> Result<(), String> {
    if lines.is_empty() {
        return Err("checks file has no commands".to_string());
    }
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return Err(format!("checks line {} is empty", i + 1));
        }
        if trimmed.starts_with('#') {
            return Err(format!("checks line {} is comment-only", i + 1));
        }
        let first = trimmed.split_whitespace().next().unwrap_or("");
        if first.is_empty() {
            return Err(format!("checks line {} has no command", i + 1));
        }
        if crate::lookup_bin_on_path(first).is_none() {
            return Err(format!(
                "checks line {}: command not found on PATH: {first}",
                i + 1
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_dir_is_empty_for_discovery() {
        let tmp = tempfile::tempdir().unwrap();
        Command::new("git")
            .args(["init"])
            .current_dir(tmp.path())
            .status()
            .unwrap();
        assert!(init_repo_is_empty_for_discovery(tmp.path()));
    }

    #[test]
    fn py_file_without_commits_is_not_empty() {
        let tmp = tempfile::tempdir().unwrap();
        Command::new("git")
            .args(["init"])
            .current_dir(tmp.path())
            .status()
            .unwrap();
        std::fs::write(tmp.path().join("foo.py"), "x = 1\n").unwrap();
        assert!(!init_repo_is_empty_for_discovery(tmp.path()));
    }

    #[test]
    fn commits_only_docs_runs_discovery() {
        let tmp = tempfile::tempdir().unwrap();
        Command::new("git")
            .args(["init"])
            .current_dir(tmp.path())
            .status()
            .unwrap();
        std::fs::write(tmp.path().join("README.md"), "hi\n").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(tmp.path())
            .status()
            .unwrap();
        Command::new("git")
            .args([
                "-c",
                "user.name=t",
                "-c",
                "user.email=t@t",
                "commit",
                "-m",
                "c",
            ])
            .current_dir(tmp.path())
            .status()
            .unwrap();
        assert!(repo_has_git_commits(tmp.path()));
        assert!(!repo_has_meaningful_source_or_tooling(tmp.path()));
        assert!(!init_repo_is_empty_for_discovery(tmp.path()));
    }

    #[test]
    fn discovery_skipped_when_checks_exist_without_force() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("lib.py"), "x = 1\n").unwrap();
        let d = init_discovery_decision(
            tmp.path(),
            InitDiscoveryRequest {
                checks_existed_before_bootstrap: true,
                force_overwrite: false,
            },
        );
        assert!(!d.run);
        assert_eq!(d.skip_reason, Some("checks already present; discovery skipped"));
    }

    #[test]
    fn discovery_runs_with_force_when_checks_existed() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("foo.py"), "x = 1\n").unwrap();
        let d = init_discovery_decision(
            tmp.path(),
            InitDiscoveryRequest {
                checks_existed_before_bootstrap: true,
                force_overwrite: true,
            },
        );
        assert!(d.run);
        assert!(d.skip_reason.is_none());
    }

    #[test]
    fn validate_checks_rejects_missing_binary() {
        assert!(
            validate_checks_command_lines(&["definitely_not_a_malvin_binary_xyz".to_string()])
                .is_err()
        );
    }

    #[test]
    fn validate_checks_rejects_comment_only() {
        assert!(validate_checks_command_lines(&["# only".to_string()]).is_err());
    }

    #[test]
    fn empty_repo_skip_implies_no_summary_phase() {
        let tmp = tempfile::tempdir().unwrap();
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(tmp.path())
            .status()
            .unwrap();
        let d = init_discovery_decision(
            tmp.path(),
            InitDiscoveryRequest {
                checks_existed_before_bootstrap: false,
                force_overwrite: false,
            },
        );
        assert!(!d.run);
        let run_summary = d.run || d.skip_reason != Some("empty repo; using builtin checks");
        assert!(!run_summary);
    }

    #[test]
    fn validate_checks_accepts_kiss_on_path() {
        if crate::lookup_bin_on_path("kiss").is_some() {
            validate_checks_command_lines(&["kiss check".to_string()]).unwrap();
        }
    }
}
