use std::path::PathBuf;

use crate::acp_post_run::{
    duplicate_safe_restore_error, merge_acp_and_timing_results,
    merge_acp_with_workspace_session_restore_and_check_abort, prefer_primary_over_secondary,
};

fn abort_result_path(dir: &tempfile::TempDir) -> PathBuf {
    let result = dir.path().join("result.md");
    std::fs::write(&result, "ABORT: stop\n").unwrap();
    result
}

fn merge_timing_ok_acp_ok_propagates_timing_err() {
    assert_eq!(
        merge_acp_and_timing_results(Ok(()), Err(std::io::Error::other("disk"))),
        Err("disk".to_string())
    );
}

fn merge_timing_ok_acp_err_drops_timing_result() {
    assert_eq!(
        merge_acp_and_timing_results(Err("acp".into()), Err(std::io::Error::other("disk"))),
        Err("acp".into())
    );
}

fn merge_both_ok() {
    assert_eq!(merge_acp_and_timing_results(Ok(()), Ok(())), Ok(()));
}

fn prefer_primary_appends_secondary_error_when_primary_fails() {
    assert_eq!(
        prefer_primary_over_secondary(
            Err("wf".into()),
            Err("restore".into()),
            "workspace session restore failed",
        ),
        Err("wf; workspace session restore failed: restore".into())
    );
}

fn prefer_primary_surfaces_secondary_when_primary_ok() {
    assert_eq!(
        prefer_primary_over_secondary(Ok(()), Err("restore".into()), "x"),
        Err("restore".into())
    );
}

fn merge_error_mentions_restore_detects_workspace_failure() {
    assert!(crate::acp_post_run::merge_error_mentions_restore(
        "workspace session restore failed: disk"
    ));
    assert!(!crate::acp_post_run::merge_error_mentions_restore(
        "unrelated"
    ));
}

fn prefer_primary_ok_when_both_ok() {
    assert_eq!(prefer_primary_over_secondary(Ok(()), Ok(()), "x"), Ok(()));
}

fn prefer_primary_surfaces_primary_when_secondary_ok() {
    assert_eq!(
        prefer_primary_over_secondary(Err("wf".into()), Ok(()), "x"),
        Err("wf".into())
    );
}

fn duplicate_safe_restore_error_does_not_repeat_restore_prefix() {
    assert_eq!(
        duplicate_safe_restore_error("wf failed; workspace session restore failed: restore")
            .as_str(),
        "wf failed; workspace session restore failed: restore"
    );
}

fn duplicate_safe_restore_error_adds_restore_prefix_when_missing() {
    assert_eq!(
        duplicate_safe_restore_error("wf failed"),
        "workspace session restore failed: wf failed"
    );
}

fn merge_with_abort_after_successful_restore() {
    let tmp = tempfile::tempdir().unwrap();
    let result = abort_result_path(&tmp);
    let work = tempfile::tempdir().unwrap();
    let empty = crate::test_utils::empty_session_dotfile_backups(work.path());
    let err = merge_acp_with_workspace_session_restore_and_check_abort(
        Ok(()),
        work.path(),
        &empty,
        &result,
    )
    .unwrap_err();
    assert_eq!(err, "ABORT: stop");
}

fn merge_with_abort_does_not_claim_restore_failed_when_restore_succeeded() {
    let tmp = tempfile::tempdir().unwrap();
    let result = abort_result_path(&tmp);
    let work = tempfile::tempdir().unwrap();
    let empty = crate::test_utils::empty_session_dotfile_backups(work.path());
    let err = merge_acp_with_workspace_session_restore_and_check_abort(
        Err("wf failed".into()),
        work.path(),
        &empty,
        &result,
    )
    .unwrap_err();
    assert!(err.contains("ABORT: stop"));
    assert!(err.contains("wf failed"));
    assert!(
        !err.contains("workspace session restore failed"),
        "restore succeeded; got: {err}"
    );
}

fn duplicate_safe_restore_error_recognizes_slot_restore_prefix() {
    let err = "wf failed; malvin_checks restore: permission denied";
    assert_eq!(duplicate_safe_restore_error(err), err);
}

fn work_dir_with_checks(
    content: &str,
) -> (tempfile::TempDir, crate::artifacts::SessionDotfileBackups) {
    let work = tempfile::tempdir().unwrap();
    crate::seed_malvin_checks(work.path(), content);
    let backups = crate::test_utils::empty_session_dotfile_backups(work.path());
    (work, backups)
}

fn merge_with_abort_combines_restore_failure() {
    let tmp = tempfile::tempdir().unwrap();
    let result = abort_result_path(&tmp);
    let (work, backups) = work_dir_with_checks("x\n");
    crate::seed_malvin_checks(work.path(), "changed\n");
    let err = merge_acp_with_workspace_session_restore_and_check_abort(
        Err("wf failed".into()),
        work.path(),
        &backups,
        &result,
    )
    .unwrap_err();
    assert!(err.contains("ABORT: stop"));
    assert!(err.contains("wf failed"));
}

#[test]
fn kiss_bundled_cli_acp_post_run_tests() {
    merge_timing_ok_acp_ok_propagates_timing_err();
    merge_timing_ok_acp_err_drops_timing_result();
    merge_both_ok();
    prefer_primary_appends_secondary_error_when_primary_fails();
    prefer_primary_surfaces_secondary_when_primary_ok();
    merge_error_mentions_restore_detects_workspace_failure();
    prefer_primary_ok_when_both_ok();
    prefer_primary_surfaces_primary_when_secondary_ok();
    duplicate_safe_restore_error_does_not_repeat_restore_prefix();
    duplicate_safe_restore_error_adds_restore_prefix_when_missing();
    merge_with_abort_after_successful_restore();
    merge_with_abort_does_not_claim_restore_failed_when_restore_succeeded();
    duplicate_safe_restore_error_recognizes_slot_restore_prefix();
    merge_with_abort_combines_restore_failure();
}
