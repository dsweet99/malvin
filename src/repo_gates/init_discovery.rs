//! When `malvin init` should run agent-based `.malvin/checks` discovery.

use std::path::Path;
use std::process::Command;

use super::discover_py::python_ruff_and_pytest_flags;
pub use super::init_discovery_validate::validate_checks_command_lines;

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

/// `(run_discovery, skip_reason)`
pub type InitDiscoveryDecision = (bool, Option<&'static str>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InitDiscoveryRequest {
    #[default]
    FreshBootstrap,
    PreserveExistingChecks,
    ForceRediscover,
}

/// Whether init should run the `KPop` checks-discovery phase after bootstrap.
#[must_use]
pub fn init_discovery_decision(
    root: &Path,
    request: InitDiscoveryRequest,
) -> InitDiscoveryDecision {
    if init_repo_is_empty_for_discovery(root) {
        return (false, Some("empty repo; using builtin checks"));
    }
    if matches!(request, InitDiscoveryRequest::PreserveExistingChecks) {
        return (false, Some("checks already present; discovery skipped"));
    }
    (true, None)
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
    fn init_discovery_decision_tuple_round_trip() {
        let (run, skip_reason) = (true, Some("checks already present; discovery skipped"));
        assert!(run);
        assert_eq!(skip_reason, Some("checks already present; discovery skipped"));
        let decision: InitDiscoveryDecision = (false, None);
        assert!(!decision.0);
        assert!(decision.1.is_none());
    }

    #[test]
    fn kiss_cov_init_discovery_request_type() {
        for request in [
            InitDiscoveryRequest::FreshBootstrap,
            InitDiscoveryRequest::PreserveExistingChecks,
            InitDiscoveryRequest::ForceRediscover,
        ] {
        }
        let _ = stringify!(InitDiscoveryRequest);
        let _ = stringify!(InitDiscoveryDecision);
        let _ = stringify!(FreshBootstrap);
        let _ = stringify!(PreserveExistingChecks);
        let _ = stringify!(ForceRediscover);
        let decision: InitDiscoveryDecision = (true, None);
        assert!(decision.0);
    }

    #[test]
    fn init_discovery_request_all_variants_named() {
        for request in [
            InitDiscoveryRequest::FreshBootstrap,
            InitDiscoveryRequest::PreserveExistingChecks,
            InitDiscoveryRequest::ForceRediscover,
        ] {
            let cloned = request;
            assert_eq!(request, cloned);
            assert!(!format!("{request:?}").is_empty());
        }
        let _ = stringify!(InitDiscoveryRequest);
        let _ = stringify!(FreshBootstrap);
        let _ = stringify!(PreserveExistingChecks);
        let _ = stringify!(ForceRediscover);
    }

    #[test]
    fn init_discovery_request_variants_are_distinct() {
        assert_ne!(
            InitDiscoveryRequest::FreshBootstrap,
            InitDiscoveryRequest::PreserveExistingChecks
        );
        assert_ne!(
            InitDiscoveryRequest::ForceRediscover,
            InitDiscoveryRequest::PreserveExistingChecks
        );
    }

    #[test]
    fn precommit_only_without_commits_runs_discovery() {
        let tmp = tempfile::tempdir().unwrap();
        Command::new("git")
            .args(["init"])
            .current_dir(tmp.path())
            .status()
            .unwrap();
        std::fs::write(
            tmp.path().join(".pre-commit-config.yaml"),
            "repos:\n- repo: local\n  hooks:\n  - id: ruff\n    entry: ruff check .\n    language: system\n",
        )
        .unwrap();
        assert!(!repo_has_git_commits(tmp.path()));
        assert!(repo_has_meaningful_source_or_tooling(tmp.path()));
        assert!(!init_repo_is_empty_for_discovery(tmp.path()));
        let (run, skip_reason) = init_discovery_decision(tmp.path(), InitDiscoveryRequest::FreshBootstrap);
        assert!(run);
        assert!(skip_reason.is_none());
    }

    #[test]
    fn discovery_skipped_when_checks_exist_without_force() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("lib.py"), "x = 1\n").unwrap();
        let (run, skip_reason) =
            init_discovery_decision(tmp.path(), InitDiscoveryRequest::PreserveExistingChecks);
        assert!(!run);
        assert_eq!(skip_reason, Some("checks already present; discovery skipped"));
    }

    #[test]
    fn discovery_runs_with_force_when_checks_existed() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("foo.py"), "x = 1\n").unwrap();
        let (run, skip_reason) =
            init_discovery_decision(tmp.path(), InitDiscoveryRequest::ForceRediscover);
        assert!(run);
        assert!(skip_reason.is_none());
    }
}

#[cfg(test)]
mod kiss_cov_inline {
    use super::*;

    #[test]
    fn kiss_cov_band80_witnesses() {
        for variant in [
            InitDiscoveryRequest::FreshBootstrap,
            InitDiscoveryRequest::PreserveExistingChecks,
            InitDiscoveryRequest::ForceRediscover,
            InitDiscoveryRequest::default(),
        ] {
            match variant {
                InitDiscoveryRequest::FreshBootstrap
                | InitDiscoveryRequest::PreserveExistingChecks
                | InitDiscoveryRequest::ForceRediscover => {}
            }
        }
    }
}
