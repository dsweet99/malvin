//! Tests for [`super::router_flow_acp_support`].

use super::{
    iteration_backups_after_router_a, router_b_template_and_label, workspace_has_valid_checks,
    RouterAInitSnapshotInput, RouterChecksSnapshotMode,
};
use crate::artifacts::{MalvinChecksBackup, SessionDotfileBackups};
use crate::prompts::{ROUTER_B_COMPLEX_MD, ROUTER_B_SIMPLE_MD};

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
fn router_b_template_and_label_selects_by_score() {
    assert_eq!(
        router_b_template_and_label(3),
        (ROUTER_B_SIMPLE_MD, "router_b_simple")
    );
    assert_eq!(
        router_b_template_and_label(4),
        (ROUTER_B_COMPLEX_MD, "router_b_complex")
    );
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
