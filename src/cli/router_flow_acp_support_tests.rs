//! Tests for [`super::router_flow_acp_support`].

use super::{
    iteration_backups_after_router_a, maybe_run_router_init, router_b_template_and_label,
    workspace_has_valid_checks, RouterAInitSnapshotInput, RouterChecksSnapshotMode,
};
use crate::artifacts::{MalvinChecksBackup, SessionDotfileBackups};
use crate::cli::error_run_log::{
    clear_command_error_run_dir, command_error_run_dir, set_command_error_run_dir,
};
use crate::prompts::ROUTER_B_MD;

#[test]
fn router_a_init_snapshot_mode_selects_refresh_only_for_coding_without_checks() {
    assert!(matches!(
        RouterAInitSnapshotInput {
            coding_task: true,
            had_checks: false,
        }
        .snapshot_mode(),
        RouterChecksSnapshotMode::RefreshAfterPossibleInit
    ));
    assert!(matches!(
        RouterAInitSnapshotInput {
            coding_task: true,
            had_checks: true,
        }
        .snapshot_mode(),
        RouterChecksSnapshotMode::KeepPreInit
    ));
    assert!(matches!(
        RouterAInitSnapshotInput {
            coding_task: false,
            had_checks: false,
        }
        .snapshot_mode(),
        RouterChecksSnapshotMode::KeepPreInit
    ));
}

#[test]
fn router_b_template_and_label_is_always_router_b() {
    assert_eq!(router_b_template_and_label(), (ROUTER_B_MD, "router_b"));
}

#[test]
fn workspace_has_valid_checks_false_when_missing() {
    let tmp = tempfile::tempdir().expect("tempdir");
    assert!(!workspace_has_valid_checks(tmp.path()).expect("checks"));
}

#[test]
fn iteration_backups_after_router_a_refreshes_when_init_may_run() {
    crate::test_utils::with_isolated_home(|workspace| {
        let pre = SessionDotfileBackups::snapshot_after_ensuring_home_config(workspace)
            .expect("snapshot");
        assert!(matches!(pre.malvin_checks, MalvinChecksBackup::Missing));
        crate::seed_malvin_checks(workspace, "true\n");
        let refreshed = iteration_backups_after_router_a(
            workspace,
            RouterChecksSnapshotMode::RefreshAfterPossibleInit,
            pre,
        )
        .expect("refresh");
        assert!(matches!(
            refreshed.malvin_checks,
            MalvinChecksBackup::Present(_)
        ));
    });
}

#[test]
fn maybe_run_router_init_restores_router_error_run_log_binding() {
    crate::test_utils::with_isolated_home(|workspace| {
        crate::test_utils::block_on_test_async(async {
            crate::seed_malvin_checks(workspace, "true\n");
            let router_dir = workspace.join("router-run");
            std::fs::create_dir_all(&router_dir).expect("router run dir");
            set_command_error_run_dir(Some(router_dir.clone()));
            let (shared, _) =
                crate::router_flow::router_flow_acp::router_flow_acp_tests::test_router_shared();
            maybe_run_router_init(workspace, &shared, true)
                .await
                .expect("init skipped when checks already valid");
            assert_eq!(command_error_run_dir(), Some(router_dir));
            clear_command_error_run_dir();
        });
    });
}

#[test]
fn iteration_backups_after_router_a_keeps_pre_init_when_checks_exist() {
    crate::test_utils::with_isolated_home(|workspace| {
        crate::seed_malvin_checks(workspace, "true\n");
        let pre = SessionDotfileBackups::snapshot_after_ensuring_home_config(workspace)
            .expect("snapshot");
        let kept = iteration_backups_after_router_a(
            workspace,
            RouterChecksSnapshotMode::KeepPreInit,
            pre.clone(),
        )
        .expect("keep");
        assert_eq!(format!("{kept:?}"), format!("{pre:?}"));
    });
}
