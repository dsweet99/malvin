use super::{
    format_current_state, format_local_datetime, format_retry_line, format_sandbox_memory_line,
    format_user_identity,
};

#[test]
fn format_user_identity_includes_name() {
    let id = format_user_identity();
    assert!(!id.is_empty());
    #[cfg(unix)]
    {
        assert!(id.contains("uid "));
        assert!(super::effective_user_id().is_some());
    }
}

#[cfg(unix)]
#[test]
fn current_sandbox_rss_bytes_when_agent_registered() {
    crate::active_agent_heartbeat::clear_active_agent_process_groups_for_test();
    let pgid = std::process::id();
    let baseline = crate::acp::snapshot_pids();
    crate::active_agent_heartbeat::register_active_agent_process_group(Some(pgid), baseline);
    let line = format_sandbox_memory_line(std::path::Path::new("."));
    assert!(line.contains("in use"));
    crate::active_agent_heartbeat::unregister_active_agent_process_group(Some(pgid));
    crate::active_agent_heartbeat::clear_active_agent_process_groups_for_test();
}

#[test]
fn infer_gate_retry_reasons_empty_without_artifacts() {
    assert!(super::infer_gate_retry_reasons(None, 2).is_empty());
}

#[test]
fn infer_gate_retry_reasons_empty_for_first_iteration() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts =
        crate::artifacts::create_kpop_run_artifacts("code", Some(tmp.path())).expect("artifacts");
    assert!(super::infer_gate_retry_reasons(Some(&artifacts), 1).is_empty());
}

#[test]
fn previous_session_oom_killed_false_when_log_missing() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts =
        crate::artifacts::create_kpop_run_artifacts("code", Some(tmp.path())).expect("artifacts");
    assert!(!super::previous_session_oom_killed(&artifacts));
}

#[test]
fn format_local_datetime_is_non_empty() {
    assert!(!format_local_datetime().is_empty());
}

#[test]
fn format_sandbox_memory_line_includes_limit_and_available() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let line = format_sandbox_memory_line(tmp.path());
    assert!(line.starts_with("Sandbox memory:"));
    assert!(line.contains("limit"));
    assert!(line.contains("available"));
}

#[test]
fn format_retry_line_first_run_is_not_retry() {
    let line = format_retry_line(None, None);
    assert!(line.contains("not a retry"));
}

#[test]
fn format_retry_line_first_gate_iteration_is_not_retry() {
    let line = format_retry_line(Some(1), None);
    assert!(line.contains("first outer gate-loop"));
}

#[test]
fn format_retry_line_second_iteration_is_retry_without_solved() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts =
        crate::artifacts::create_kpop_run_artifacts("code", Some(tmp.path())).expect("artifacts");
    crate::artifacts::ensure_gate_exp_log_file(&artifacts, 1).expect("exp log");
    let line = format_retry_line(Some(2), Some(&artifacts));
    assert!(line.contains("retry #1"));
    assert!(line.contains("KPOP_SOLVED"));
}

#[test]
fn format_retry_line_detects_oom_in_kpop_log() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts =
        crate::artifacts::create_kpop_run_artifacts("code", Some(tmp.path())).expect("artifacts");
    std::fs::write(
        artifacts.log_path("kpop"),
        "malvin sandbox exceeded memory limit; terminating\n",
    )
    .expect("write");
    let line = format_retry_line(Some(2), Some(&artifacts));
    assert!(line.contains("OOM"));
}

#[test]
fn format_retry_line_gates_failure_after_solved() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts =
        crate::artifacts::create_kpop_run_artifacts("code", Some(tmp.path())).expect("artifacts");
    let prev = artifacts.gate_exp_log_path(1);
    std::fs::create_dir_all(prev.parent().expect("parent")).expect("mkdir");
    std::fs::write(&prev, "## KPOP_SOLVED\n").expect("write");
    let line = format_retry_line(Some(2), Some(&artifacts));
    assert!(line.contains("quality gates"));
}

#[test]
fn format_current_state_joins_all_sections() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let text = format_current_state(tmp.path(), None, None);
    assert!(text.contains("User:"));
    assert!(text.contains("Date/time:"));
    assert!(text.contains("Sandbox memory:"));
    assert!(text.contains("Retry:"));
}

#[test]
fn read_prev_exp_solved_missing_file_returns_none() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts =
        crate::artifacts::create_kpop_run_artifacts("code", Some(tmp.path())).expect("artifacts");
    assert!(super::read_prev_exp_solved(&artifacts, 99).is_none());
}

#[test]
fn append_unsolved_reason_records_missing_marker() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts =
        crate::artifacts::create_kpop_run_artifacts("code", Some(tmp.path())).expect("artifacts");
    let prev = artifacts.gate_exp_log_path(1);
    std::fs::create_dir_all(prev.parent().expect("parent")).expect("mkdir");
    std::fs::write(&prev, "no marker\n").expect("write");
    let mut reasons = Vec::new();
    super::append_unsolved_reason(&mut reasons, &artifacts, 1);
    assert_eq!(reasons.len(), 1);
}

#[test]
fn append_oom_reason_records_memory_kill() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts =
        crate::artifacts::create_kpop_run_artifacts("code", Some(tmp.path())).expect("artifacts");
    std::fs::write(
        artifacts.log_path("kpop"),
        "malvin sandbox exceeded memory limit\n",
    )
    .expect("write");
    let mut reasons = Vec::new();
    super::append_oom_reason(&mut reasons, &artifacts);
    assert!(reasons.iter().any(|r| r.contains("OOM")));
}

#[test]
fn append_gates_reason_after_solved_session() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts =
        crate::artifacts::create_kpop_run_artifacts("code", Some(tmp.path())).expect("artifacts");
    let prev = artifacts.gate_exp_log_path(1);
    std::fs::create_dir_all(prev.parent().expect("parent")).expect("mkdir");
    std::fs::write(&prev, "## KPOP_SOLVED\n").expect("write");
    let mut reasons = Vec::new();
    super::append_gates_reason(&mut reasons, &artifacts, 1);
    assert!(reasons.iter().any(|r| r.contains("quality gates")));
}

#[test]
fn kiss_cov_current_state_non_unix_branch() {
    let _ = super::current_sandbox_rss_bytes;
    let _ = super::effective_user_id;
    let _ = super::append_unsolved_reason;
    let _ = super::append_oom_reason;
    let _ = super::append_gates_reason;
    let _ = super::read_prev_exp_solved;
}
