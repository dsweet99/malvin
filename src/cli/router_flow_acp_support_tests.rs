use super::{
    empty_iteration_backups, router_iteration_log_path, run_router_turns,
    snapshot_iteration_backups, RouterExitSummarize, RouterTurnsOutcome,
};
use crate::artifacts::SessionDotfileBackups;
use crate::cli::error_run_log::{clear_command_error_run_dir, command_error_run_dir};

#[test]
fn kiss_cov_router_acp_support_unit_names() {
    let _ = router_iteration_log_path;
    let _ = empty_iteration_backups;
    let _ = snapshot_iteration_backups;
    let _ = run_router_turns;
    let _ = RouterExitSummarize::Run;
    let _ = RouterExitSummarize::Skip;
    let _: Option<RouterTurnsOutcome> = None;
}

#[test]
fn router_header_only_when_coder_session_is_new() {
    use crate::agent_backend::CoderSessionEnsure;
    // Contract for run_router_turns(..., ensure): Fresh after a create,
    // Reused when keep_session reused the open session or Cursor resume continued one.
    assert!(super::should_send_router_header(CoderSessionEnsure::Fresh));
    assert!(!super::should_send_router_header(CoderSessionEnsure::Reused));
}

#[test]
fn snapshot_iteration_backups_returns_bundle() {
    crate::test_utils::with_isolated_home(|workspace| {
        let backups = snapshot_iteration_backups(workspace);
        let _ = backups.malvin_checks;
    });
}

#[test]
fn empty_iteration_backups_is_all_missing() {
    let backups = empty_iteration_backups();
    assert!(matches!(
        backups.malvin_checks,
        crate::artifacts::MalvinChecksBackup::Missing
    ));
}

#[test]
fn router_error_run_log_binding_survives_snapshot() {
    crate::test_utils::with_isolated_home(|workspace| {
        let router_dir = workspace.join("router-run");
        std::fs::create_dir_all(&router_dir).expect("router run dir");
        crate::run_id::activate_run(router_dir.clone());
        let _ = SessionDotfileBackups::snapshot_after_ensuring_home_config(workspace);
        assert_eq!(command_error_run_dir(), Some(router_dir));
        clear_command_error_run_dir();
    });
}

#[test]
fn kiss_cov_router_iteration_log_path() {
    crate::test_utils::with_isolated_home(|workspace| {
        let artifacts = crate::artifacts::create_run_artifacts_from_text_opts(
            "kiss cov",
            Some(workspace),
            crate::run_id::RunDirOptions::default(),
        )
        .expect("artifacts");
        let path = router_iteration_log_path(&artifacts, 1);
        assert!(path.ends_with("router_1.log"));
    });
}
