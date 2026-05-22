use std::path::{Path, PathBuf};

use crate::acp_post_run::{
    duplicate_safe_restore_error, merge_acp_and_timing_results,
    merge_acp_with_workspace_session_restore_and_check_abort, prefer_primary_over_secondary,
};

fn abort_result_path(dir: &tempfile::TempDir) -> PathBuf {
    let result = dir.path().join("result.md");
    std::fs::write(&result, "ABORT: stop\n").unwrap();
    result
}

fn empty_session_backups(work: &Path) -> crate::artifacts::SessionDotfileBackups {
    crate::artifacts::SessionDotfileBackups::from_parts(
        crate::artifacts::backup_workspace_kissconfig_if_present(work).unwrap(),
        crate::artifacts::backup_workspace_malvin_checks_if_present(work).unwrap(),
        crate::artifacts::backup_workspace_kissignore_if_present(work).unwrap(),
    )
}

fn smoke_agent_client() -> crate::acp::AgentClient {
    use crate::acp::{AgentClient, AgentIoOptions};
    AgentClient::new(
        "m".into(),
        AgentIoOptions {
            force: false,
            no_tee: true,
            raw_output: true,
            show_thoughts_on_stdout: false,
            emit_stdout_markdown: false,
            log_full_outgoing_prompts: false,
        },
    )
}

#[test]
fn emit_run_timing_json_only_after_acp_writes_json() {
    use std::sync::{Arc, Mutex};

    use crate::acp_post_run::emit_run_timing_json_only_after_acp;
    use crate::run_timing::RUN_TIMING_JSON_FILE;

    let mut client = smoke_agent_client();
    let tmp = tempfile::tempdir().expect("tempdir");
    let run_dir = tmp.path().join("run");
    std::fs::create_dir_all(&run_dir).expect("mkdir");
    let timing = Arc::new(Mutex::new(crate::run_timing::RunTiming::default()));
    emit_run_timing_json_only_after_acp(&mut client, &run_dir, &timing, Ok(()))
        .expect("emit timing");
    assert!(run_dir.join(RUN_TIMING_JSON_FILE).is_file());
}

#[test]
fn emit_run_timing_after_acp_writes_json() {
    use std::sync::{Arc, Mutex};
    use std::time::Instant;

    use crate::acp_post_run::emit_run_timing_after_acp;
    use crate::run_timing::RUN_TIMING_JSON_FILE;

    let mut client = smoke_agent_client();
    let tmp = tempfile::tempdir().expect("tempdir");
    let run_dir = tmp.path().join("run");
    std::fs::create_dir_all(&run_dir).expect("mkdir");
    let timing = Arc::new(Mutex::new(crate::run_timing::RunTiming::default()));
    {
        let mut g = timing.lock().expect("timing lock");
        g.mark_wall_start(Instant::now());
        g.mark_wall_end(Instant::now());
    }
    emit_run_timing_after_acp(&mut client, &run_dir, &timing, Ok(())).expect("emit timing");
    assert!(run_dir.join(RUN_TIMING_JSON_FILE).is_file());
}

#[test]
fn merge_timing_ok_acp_ok_propagates_timing_err() {
    assert_eq!(
        merge_acp_and_timing_results(Ok(()), Err(std::io::Error::other("disk"))),
        Err("disk".to_string())
    );
}

#[test]
fn merge_timing_ok_acp_err_drops_timing_result() {
    assert_eq!(
        merge_acp_and_timing_results(Err("acp".into()), Err(std::io::Error::other("disk"))),
        Err("acp".into())
    );
}

#[test]
fn merge_both_ok() {
    assert_eq!(merge_acp_and_timing_results(Ok(()), Ok(())), Ok(()));
}

#[test]
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

#[test]
fn prefer_primary_surfaces_secondary_when_primary_ok() {
    assert_eq!(
        prefer_primary_over_secondary(Ok(()), Err("restore".into()), "x"),
        Err("restore".into())
    );
}

#[test]
fn merge_error_mentions_restore_detects_workspace_failure() {
    assert!(crate::acp_post_run::merge_error_mentions_restore(
        "workspace session restore failed: disk"
    ));
    assert!(!crate::acp_post_run::merge_error_mentions_restore(
        "unrelated"
    ));
}

#[test]
fn prefer_primary_ok_when_both_ok() {
    assert_eq!(prefer_primary_over_secondary(Ok(()), Ok(()), "x"), Ok(()));
}

#[test]
fn prefer_primary_surfaces_primary_when_secondary_ok() {
    assert_eq!(
        prefer_primary_over_secondary(Err("wf".into()), Ok(()), "x"),
        Err("wf".into())
    );
}

#[test]
fn duplicate_safe_restore_error_does_not_repeat_restore_prefix() {
    assert_eq!(
        duplicate_safe_restore_error("wf failed; workspace session restore failed: restore")
            .as_str(),
        "wf failed; workspace session restore failed: restore"
    );
}

#[test]
fn duplicate_safe_restore_error_adds_restore_prefix_when_missing() {
    assert_eq!(
        duplicate_safe_restore_error("wf failed"),
        "workspace session restore failed: wf failed"
    );
}

#[test]
fn merge_with_abort_after_successful_restore() {
    let tmp = tempfile::tempdir().unwrap();
    let result = abort_result_path(&tmp);
    let work = tempfile::tempdir().unwrap();
    let empty = empty_session_backups(work.path());
    let err = merge_acp_with_workspace_session_restore_and_check_abort(
        Ok(()),
        work.path(),
        &empty,
        &result,
    )
    .unwrap_err();
    assert_eq!(err, "ABORT: stop");
}

#[test]
fn merge_with_abort_does_not_claim_restore_failed_when_restore_succeeded() {
    let tmp = tempfile::tempdir().unwrap();
    let result = abort_result_path(&tmp);
    let work = tempfile::tempdir().unwrap();
    let empty = empty_session_backups(work.path());
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

#[test]
fn duplicate_safe_restore_error_recognizes_slot_restore_prefix() {
    let err = "wf failed; malvin_checks restore: permission denied";
    assert_eq!(duplicate_safe_restore_error(err), err);
}

#[test]
fn merge_with_abort_combines_restore_failure() {
    let tmp = tempfile::tempdir().unwrap();
    let result = abort_result_path(&tmp);
    let work = tempfile::tempdir().unwrap();
    std::fs::write(work.path().join(".malvin_checks"), "x\n").unwrap();
    let backups = empty_session_backups(work.path());
    std::fs::write(work.path().join(".malvin_checks"), "changed\n").unwrap();
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
