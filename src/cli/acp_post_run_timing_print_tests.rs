use crate::acp_post_run::merge_acp_restore_check_abort_then_print_timing;
use crate::artifacts::RunArtifacts;
use crate::output::{STDOUT_LOG_TEST_LOCK, set_stdout_log_path};
use crate::run_timing::RunTiming;
use crate::llm_transport::ResponseUsage;
use std::time::Instant;

fn empty_artifacts(work: &tempfile::TempDir) -> (crate::artifacts::SessionDotfileBackups, RunArtifacts) {
    let empty = crate::test_utils::empty_session_dotfile_backups(work.path());
    let artifacts =
        crate::artifacts::create_kpop_run_artifacts("code", Some(work.path())).expect("artifacts");
    (empty, artifacts)
}

fn seed_timing_json_with_cost(run_dir: &std::path::Path) {
    let timing = RunTiming::new_arc();
    {
        let mut g = timing.lock().unwrap();
        g.mark_wall_start(Instant::now());
        g.mark_wall_end(Instant::now());
        g.record_completion_cost(&ResponseUsage {
            prompt_tokens: None,
            completion_tokens: None,
            total_tokens: Some(1),
            cost: Some(0.042),
        });
    }
    crate::run_timing::finalize_run_timing_json_only(run_dir, &timing).expect("json only");
}

fn assert_timing_and_cost_in_log(log: &str) {
    assert!(
        log.contains("TIMING:"),
        "router/kpop finish path must print TIMING; log={log:?}"
    );
    assert!(
        log.contains("COST:"),
        "router/kpop finish path must print COST; log={log:?}"
    );
    assert!(
        !log.contains("TOKENS:"),
        "TOKENS footnote must not appear; log={log:?}"
    );
    let timing_pos = log.find("TIMING:").expect("TIMING");
    let cost_pos = log.find("COST:").expect("COST");
    assert!(
        timing_pos < cost_pos,
        "footnote order TIMING < COST; log={log:?}"
    );
    assert!(
        log.contains("steps ="),
        "combined COST line must include token fields; log={log:?}"
    );
}

#[test]
fn merge_restore_check_abort_then_print_timing_noops_without_json() {
    let work = tempfile::tempdir().unwrap();
    let (empty, artifacts) = empty_artifacts(&work);
    merge_acp_restore_check_abort_then_print_timing(Ok(()), &artifacts, &empty).expect("merge");
}

#[test]
fn merge_restore_check_abort_then_print_timing_emits_timing_and_cost() {
    let _stdout = STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    let work = tempfile::tempdir().unwrap();
    let (empty, artifacts) = empty_artifacts(&work);
    seed_timing_json_with_cost(&artifacts.run_dir);

    let log_path = artifacts.run_dir.join("stdout.log");
    set_stdout_log_path(Some(log_path.clone()));
    merge_acp_restore_check_abort_then_print_timing(Ok(()), &artifacts, &empty).expect("merge");
    set_stdout_log_path(None);

    let log = std::fs::read_to_string(&log_path).unwrap_or_default();
    assert_timing_and_cost_in_log(&log);
}
