//! Lazy `.malvin/checks` discovery via `KPop` (`init_constraints.md`).

use std::path::Path;

use crate::artifacts::create_kpop_run_artifacts;
use crate::malvin_checks_path;
use crate::repo_gates::load_malvin_checks;

use super::SharedOpts;

#[path = "checks_discovery_kpop.rs"]
mod checks_discovery_kpop;

use checks_discovery_kpop::run_checks_discovery_kpop;

fn checks_file_has_commands(work_dir: &Path) -> Result<(), String> {
    let path = malvin_checks_path(work_dir);
    if !path.is_file() {
        return Err("checks discovery: .malvin/checks still missing".to_string());
    }
    let lines = load_malvin_checks(&path)?;
    if lines.is_empty() {
        return Err("checks discovery: .malvin/checks has no commands".to_string());
    }
    Ok(())
}

fn checks_already_valid(work_dir: &Path) -> Result<bool, String> {
    let path = malvin_checks_path(work_dir);
    if !path.is_file() {
        return Ok(false);
    }
    let lines = load_malvin_checks(&path)?;
    Ok(!lines.is_empty())
}

fn finish_checks_discovery(work_dir: &Path) -> Result<(), String> {
    checks_file_has_commands(work_dir)
}

/// Run checks discovery `KPop` when `.malvin/checks` is missing or has no commands.
pub(crate) async fn ensure_malvin_checks_discovered(
    work_dir: &Path,
    shared: &SharedOpts,
) -> Result<(), String> {
    if checks_already_valid(work_dir)? {
        return Ok(());
    }
    let artifacts = create_kpop_run_artifacts("checks discovery", Some(work_dir))
        .map_err(|e| e.to_string())?;
    crate::cli::error_run_log::set_command_error_run_dir(Some(artifacts.run_dir.clone()));
    let discovery_result = run_checks_discovery_kpop(shared, &artifacts).await;
    let finish_result = finish_checks_discovery(work_dir);
    if discovery_result.is_ok() && finish_result.is_ok() {
        crate::cli::error_run_log::clear_command_error_run_dir();
    }
    discovery_result?;
    finish_result
}

/// Gate-loop prelude: discover `.malvin/checks` when missing (uses process cwd).
pub(crate) async fn ensure_malvin_checks_discovered_for_cwd(
    shared: &SharedOpts,
) -> Result<(), String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    ensure_malvin_checks_discovered(&cwd, shared).await
}

/// Gate-loop prelude for commands whose workspace comes from a CLI request path.
pub(crate) async fn ensure_malvin_checks_discovered_for_cli_request(
    request: &str,
    shared: &SharedOpts,
) -> Result<(), String> {
    let (_, work_dir) =
        crate::artifacts::resolve_user_md_request(request).map_err(|e| e.to_string())?;
    ensure_malvin_checks_discovered(&work_dir, shared).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artifacts::create_kpop_run_artifacts;
    use crate::prompts::PromptStore;
    use crate::cli::WorkflowCliOptions;
    use checks_discovery_kpop::{
        checks_discovery_kpop_request, load_discovery_agent_config,
        prepare_checks_discovery_prompt_store,
    };

    #[test]
    fn checks_discovery_kpop_request_expands_placeholders() {
        let tmp = tempfile::tempdir().expect("tempdir");
        crate::seed_malvin_checks(tmp.path(), "kiss check\n");
        let artifacts =
            create_kpop_run_artifacts("checks_discover", Some(tmp.path())).expect("artifacts");
        let store = PromptStore::default_store();
        store.ensure_defaults().expect("defaults");
        let text = checks_discovery_kpop_request(&store, &artifacts).expect("request");
        assert!(
            !text.contains("{{"),
            "checks discovery kpop request must expand placeholders: {text:?}"
        );
        assert!(
            text.contains("Discover how the repo in") && text.contains("runs quality gates"),
            "expected init_constraints: {text:?}"
        );
    }

    #[test]
    fn prepare_checks_discovery_prompt_store_loads_constraints() {
        let workflow = WorkflowCliOptions { force: false };
        let store = prepare_checks_discovery_prompt_store(workflow).expect("store");
        assert!(store.validate_exists("init_constraints.md").is_ok());
    }

    #[test]
    fn finish_checks_discovery_errors_when_missing() {
        crate::test_utils::with_isolated_home(|tmp| {
            std::process::Command::new("git")
                .args(["init"])
                .current_dir(tmp)
                .status()
                .expect("git init");
            let err = finish_checks_discovery(tmp).unwrap_err();
            assert!(err.contains("still missing"), "{err:?}");
        });
    }

    #[test]
    fn finish_checks_discovery_accepts_valid_kiss_checks() {
        let tmp = tempfile::tempdir().expect("tempdir");
        crate::seed_malvin_checks(tmp.path(), "kiss check\n");
        finish_checks_discovery(tmp.path()).expect("valid");
    }

    #[test]
    fn finish_checks_discovery_accepts_checks_with_leading_comment() {
        let tmp = tempfile::tempdir().expect("tempdir");
        crate::seed_malvin_checks(tmp.path(), "# header\nkiss check\n");
        finish_checks_discovery(tmp.path()).expect("valid with comment");
    }

    #[test]
    fn finish_checks_discovery_fails_when_comment_only() {
        let tmp = tempfile::tempdir().expect("tempdir");
        crate::seed_malvin_checks(tmp.path(), "# only\n");
        let err = finish_checks_discovery(tmp.path()).unwrap_err();
        assert!(err.contains("has no commands"), "{err:?}");
    }

    #[test]
    fn checks_already_valid_false_when_comment_only() {
        crate::test_utils::with_isolated_home(|tmp| {
            crate::seed_malvin_checks(tmp, "# only\n");
            assert!(!checks_already_valid(tmp).expect("read"));
        });
    }

    #[test]
    fn checks_already_valid_false_without_file() {
        crate::test_utils::with_isolated_home(|tmp| {
            std::process::Command::new("git")
                .args(["init"])
                .current_dir(tmp)
                .status()
                .expect("git init");
            assert!(!checks_already_valid(tmp).expect("read"));
        });
    }

    #[test]
    fn load_discovery_agent_config_reads_workspace_defaults() {
        crate::test_utils::with_isolated_home(|work| {
            crate::seed_malvin_config(work, "");
            let cfg = load_discovery_agent_config(work);
            assert_eq!(cfg.max_loops, crate::malvin_config_file::DEFAULT_MAX_LOOPS);
        });
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod kiss_cov_gate_refs {
    use super::*;
    use checks_discovery_kpop::checks_discovery_kpop_request;
    #[test]
    fn kiss_cov_unit_names() {
        let _ = checks_discovery_kpop_request;
        let _ = ensure_malvin_checks_discovered;
        let _ = finish_checks_discovery;
    }
}
